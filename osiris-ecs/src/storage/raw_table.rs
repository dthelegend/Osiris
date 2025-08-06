use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::ops::{Deref, DerefMut};
use frunk::labelled::chars::u;
use crate::storage::type_data::{DynamicBundle, TypeMetadata};

pub struct RawTable {
    data: NonNull<u8>,
    capacity: usize,
    // non-owning pointers to the data
    rows: RowInfo,
}

impl RawTable {
    pub fn new(type_infos: impl IntoIterator<Item = TypeMetadata>) -> Self {
        Self {
            data: NonNull::dangling(),
            capacity: 0,
            rows: RowInfo::new(type_infos)
        }
    }

    pub fn new_unchecked(type_infos: impl IntoIterator<Item = TypeMetadata>) -> Self {
        Self {
            data: NonNull::dangling(),
            capacity: 0,
            rows: RowInfo::new_unchecked(type_infos)
        }
    }

    pub unsafe fn from_raw_parts(data: NonNull<u8>, capacity: usize, columns: RowInfo) -> Self {
        Self {
            data,
            capacity,
            rows: columns
        }
    }

    // Ensure this table can store at least capacity
    pub fn reserve(&mut self, total_capacity: usize) {
        if total_capacity > self.capacity {
            self.grow(total_capacity - self.capacity);
        }
    }

    pub fn grow(&mut self, min_additional_capacity: usize) {
        self.grow_exact(self.capacity.max(min_additional_capacity))
    }

    fn grow_exact(&mut self, additional_capacity: usize) {
        let mut new_layout = Layout::new::<()>();
        let mut old_layout = Layout::new::<()>();
        let mut offsets: Box<[usize]> = std::iter::repeat(0).take(self.rows.len()).collect();

        for ((TypeMetadata { layout, .. }, _), offset) in self.rows.iter().zip(offsets.iter_mut()) {
            (new_layout, *offset) = layout.repeat(self.capacity + additional_capacity)
                .and_then(|(array_layout, _stride)| new_layout.extend(array_layout))
                .expect("Could not construct new layout");
            (old_layout, _) = layout.repeat(self.capacity).and_then(|(array_layout, _stride)| old_layout.extend(array_layout))
                .expect("Could not construct old layout");
        }
        new_layout = new_layout.pad_to_align();
        old_layout = old_layout.pad_to_align();

        // additional_capacity > 0 therefore, the new layout must be of greater or equal size to the old layout
        debug_assert!(new_layout.size() >= old_layout.size(), "The new layout must be of greater or equal size to the old layout!");

        let new_data = if new_layout.size() > 0 {
            let raw_data = unsafe { alloc(new_layout) };
            if raw_data.is_null() {
                handle_alloc_error(new_layout);
            }
            // SAFETY just checked this invariant
            unsafe { NonNull::new_unchecked(raw_data) }
        } else {
            new_layout.dangling()
        };

        for (( TypeMetadata { layout, .. }, ptr ), offset) in self.rows.iter_mut().zip(offsets.into_iter()).rev() {
            let ptr_in_new_data = unsafe { new_data.add(offset) };
            unsafe { std::ptr::copy_nonoverlapping(ptr.as_ptr(), ptr_in_new_data.as_ptr(), self.capacity * layout.pad_to_align().size()) };
            *ptr = ptr_in_new_data;
        }
        let old_data = std::mem::replace(&mut self.data, new_data);
        if old_layout.size() > 0 { unsafe { dealloc(old_data.as_ptr(), old_layout); } }
    }
    
    pub fn type_ids(&self) -> impl Iterator<Item = TypeId> {
        self.rows.iter().map(|&(TypeMetadata{ id, .. }, _)| id)
    }
    
    pub fn put_column(&self, idx: usize, mut f: impl FnMut(*mut u8, TypeId)) {
        for (TypeMetadata { layout, id, .. }, data_ptr) in self.rows.iter() {
            unsafe { f(data_ptr.add(layout.pad_to_align().size() * idx).as_mut(), *id) }
        }
    }

    pub unsafe fn drop_column(&self, idx: usize) {
        for (TypeMetadata { layout, drop, .. }, data_ptr) in self.rows.iter() {
            unsafe { drop(data_ptr.add(layout.pad_to_align().size() * idx).as_ptr()) }
        }
    }

    pub unsafe fn swap_columns(&self, idx_a: usize, idx_b: usize) {
        if idx_a != idx_b {
            for (TypeMetadata { layout, .. }, data_ptr) in self.rows.iter() {
                unsafe {
                    let ptr_a = data_ptr.add(layout.pad_to_align().size() * idx_a);
                    let ptr_b = data_ptr.add(layout.pad_to_align().size() * idx_b);
                    std::ptr::swap_nonoverlapping(ptr_a.as_ptr(), ptr_b.as_ptr(), layout.size());
                }
            }
        }
    }

    pub unsafe fn clear(&mut self) {
        let data = std::mem::replace(&mut self.data, NonNull::dangling());
        let mut current_layout = Layout::new::<()>();

        for (TypeMetadata { layout, .. }, ptr) in self.rows.iter_mut() {
            (current_layout, _) = layout.repeat(self.capacity).and_then(|(array_layout, _stride)| layout.extend(array_layout))
                .expect("Could not construct current layout");
            *ptr = layout.dangling();
        }
        current_layout = current_layout.pad_to_align();
        self.capacity = 0;
        if current_layout.size() > 0 { dealloc(data.as_ptr(), current_layout); }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

struct ColumnIter<'a> {
    _marker: PhantomData<&'a RawTable>,
    rows: Box<[(TypeMetadata, NonNull<u8>)]>,
    remaining_len: usize,
}

impl Drop for RawTable {
    fn drop(&mut self) {
        let mut final_layout = Layout::new::<()>();

        for (TypeMetadata { layout, .. }, .. ) in self.rows.iter() {
            (final_layout, _) = layout.repeat(self.capacity).and_then(|(array_layout, _stride)| layout.extend(array_layout))
                .expect("Could not construct old layout");
        }

        final_layout = final_layout.pad_to_align();

        let data = std::mem::replace(&mut self.data, final_layout.dangling());
        if final_layout.size() > 0 { unsafe { dealloc(data.as_ptr(), final_layout) } }
    }
}


#[repr(transparent)]
#[derive(Clone)]
pub struct RowInfo(Box<[(TypeMetadata, NonNull<u8>)]>);

impl RowInfo {
    pub fn new(type_metadata: impl IntoIterator<Item = TypeMetadata>) -> Self {
        let mut inner: Box<[(TypeMetadata, NonNull<u8>)]> = type_metadata.into_iter().map(|metadata| {
            let ptr = metadata.layout.dangling();
            (metadata, ptr)
        }).collect();
        inner.sort_unstable_by_key(|&(metadata, _)| metadata);
        assert!({
            // assert all items are unique
            inner.windows(2).all(|w| w[0] != w[1])
        }, "All item types in a row must be unique!");
        Self(inner)
    }

    pub fn new_unchecked(type_metadata: impl IntoIterator<Item = TypeMetadata>) -> Self {
        Self(
            type_metadata.into_iter().map(|metadata| {
                let ptr = metadata.layout.dangling();
                (metadata, ptr)
            }).collect()
        )
    }

    pub fn search_dynamic(&self, type_id: TypeId) -> Option<(TypeMetadata, NonNull<u8>)> {
        self.0.binary_search_by_key(&type_id, |(metadata, ptr)| metadata.id).ok().map(|idx| self.0[idx])
    }

    pub fn search<T: 'static>(&self) -> Option<(TypeMetadata, NonNull<u8>)> {
        self.search_dynamic(TypeId::of::<T>())
    }
}

impl Deref for RowInfo {
    type Target = [(TypeMetadata, NonNull<u8>)];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RowInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// TableViews are non-owning views of a table
struct TableView<'a> {
    _marker: PhantomData<&'a RawTable>,
    row_info: RowInfo
}

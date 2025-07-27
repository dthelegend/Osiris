use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ops::Index;
use std::ptr::NonNull;
use crate::storage::type_data::{SortedTypeDataArray, TypeData, TypeMetadata};

pub struct RawTable<const N: usize> {
    data: NonNull<u8>,
    capacity: usize,
    // non-owning pointers to the data
    sorted_type_infos: SortedTypeDataArray<N>,
}

impl<const N: usize> RawTable<N> {
    pub fn new(type_infos: [TypeMetadata; N]) -> Self {
        Self {
            data: NonNull::dangling(),
            capacity: 0,
            sorted_type_infos: SortedTypeDataArray::new(type_infos)
        }
    }

    pub unsafe fn from_raw_parts(data: NonNull<u8>, capacity: usize, sorted_type_infos: SortedTypeDataArray<N>) -> Self {
        Self {
            data,
            capacity,
            sorted_type_infos
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
        let mut offsets = [0; N];

        for (TypeData { metadata: TypeMetadata { layout, .. }, .. }, offset) in self.sorted_type_infos.iter().zip(offsets.iter_mut()) {
            (new_layout, *offset) = layout.repeat(self.capacity + additional_capacity)
                .and_then(|(array_layout, _stride)| new_layout.extend(array_layout))
                .expect("Could not construct new layout");
            (old_layout, _) = layout.repeat(self.capacity).and_then(|(array_layout, _stride)| old_layout.extend(array_layout))
                .expect("Could not construct old layout");
        }
        new_layout = new_layout.pad_to_align();

        // additional_capacity > 0 therefore, the new layout must be of greater or equal size to the old layout
        debug_assert!(new_layout.size() >= old_layout.size(), "The new layout must be of greater or equal size to the old layout!");

        let new_data = if new_layout.size() > 0 {
            let raw_data = unsafe { alloc(new_layout) };
            if raw_data.is_null() {
                handle_alloc_error(new_layout);
            }
            /// SAFETY just checked this invariant
            unsafe { NonNull::new_unchecked(raw_data) }
        } else {
            new_layout.dangling()
        };

        for (TypeData { metadata: TypeMetadata { layout, .. }, ptr }, offset) in self.sorted_type_infos.iter_mut().zip(offsets.into_iter()).rev() {
            let ptr_in_new_data = unsafe { new_data.add(offset) };
            unsafe { std::ptr::copy_nonoverlapping(ptr.as_ptr(), ptr_in_new_data.as_ptr(), self.capacity * layout.pad_to_align().size()) };
            *ptr = ptr_in_new_data;
        }
        let old_data = std::mem::replace(&mut self.data, new_data);
        if old_layout.size() > 0 { unsafe { dealloc(old_data.as_ptr(), old_layout); } }
    }

    fn ptr_for_id(&self, type_id: TypeId) -> Option<NonNull<u8>> {
        self.sorted_type_infos.search_dynamic(type_id).map(|x| x.ptr)
    }

    fn ptr_for_type<T: 'static>(&self) -> Option<NonNull<u8>> {
        self.sorted_type_infos.search::<T>().map(|x| x.ptr)
    }

    pub fn slice_for_type<T: 'static>(&self) -> Option<&[MaybeUninit<T>]> {
        self.ptr_for_type::<T>().map(|ptr| unsafe { std::slice::from_raw_parts(ptr.as_ptr() as *mut MaybeUninit<T>, self.capacity) })
    }

    pub fn mut_slice_for_type<T: 'static>(&self) -> Option<&mut [MaybeUninit<T>]> {
        self.ptr_for_type::<T>().map(|ptr| unsafe { std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut MaybeUninit<T>, self.capacity) })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

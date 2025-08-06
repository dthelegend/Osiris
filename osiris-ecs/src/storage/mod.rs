use frunk::labelled::chars::B;
use crate::storage::raw_table::RawTable;
use crate::storage::type_data::{DynamicBundle, DynamicBundleIter, TypeMetadata};

mod type_data;
mod raw_table;

// We can't quite type mark this because we need to be able to construct a homogenous list of Tables :(
struct Table {
    buf: RawTable,
    len: usize,
}

impl Table {
    // -- INSTANTIATION -- //
    pub fn new(types: impl IntoIterator<Item = TypeMetadata>) -> Self {
        Self {
            buf: RawTable::new(types),
            len: 0,
        }
    }

    pub fn new_from_bundle<B: DynamicBundle>() -> Self {
        Self {
            buf: RawTable::new_unchecked(B::type_ids()),
            len: 0,
        }
    }

    // --- META OPERATIONS --- //
    pub fn len(&self) -> usize { self.len }
    pub fn capacity(&self) -> usize { self.buf.capacity() }
    pub fn empty(&self) -> bool { self.len == 0 }

    pub fn reserve(&mut self, capacity: usize) {
        self.buf.reserve(capacity);
    }

    pub fn clear(&mut self) {
        for idx in 0..self.len {
            unsafe { self.buf.drop_column(idx) }
        }
        unsafe { self.buf.clear(); }
    }

    // --- SINGLE OPERATIONS --- //
    
    fn is_bundle_compatible<B: DynamicBundle>(&self) -> bool {
        self.buf.type_ids().zip(B::type_ids()).all(| (a, b )| a == b)
    }

    // unchecked bundle operation primitive
    unsafe fn put_column_unchecked<B: DynamicBundle>(&self, idx: usize, data: B) {
        unsafe {
            self.buf.put_column(idx, |dst_ptr, id| {

            });
            data.put(| src_ptr, src_id | {
                let (TypeMetadata { id: dst_id, layout, ..} , dst_ptr) = column.next().expect("Compatible Bundles must have same number of type ids as table");
                assert_eq!(src_id, dst_id, "Compatible bundles must have types identically ordered");
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, layout.size());
            });
        }
    }

    // unchecked bundle operation primitive
    unsafe fn take_column_unchecked<B: DynamicBundle>(&self, idx: usize) -> B {
        unsafe {
            let mut column = self.buf.with(idx);
            B::take(| dst_ptr, dst_id | {
                let (TypeMetadata { id: src_id, layout, ..} , src_ptr) = column.next().expect("Bundle must have same type ids as table");
                assert_eq!(src_id, dst_id, "Compatible bundles must have types identically ordered");
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, layout.size());
            })
        }
    }

    // Add an element to the table
    unsafe fn push_unchecked<B: DynamicBundle>(&mut self, data: B) {
        self.reserve(self.len + 1);

        unsafe { self.put_column_unchecked(self.len, data); }

        self.len += 1;
    }

    pub fn push<B: DynamicBundle>(&mut self, data: B) {
        assert!(self.is_bundle_compatible::<B>());
        unsafe { self.push_unchecked(data) }
    }

    pub fn insert_at<B: DynamicBundle>(&mut self, idx: usize, data: B) -> B {
        assert!(idx < self.len);
        assert!(self.is_bundle_compatible::<B>());
        unsafe {
            let output = self.take_column_unchecked(idx);
            self.put_column_unchecked(idx, data);
            output
        }
    }

    // We let the outer scope drop this bundle
    fn swap_remove<B: DynamicBundle>(&mut self, idx: usize) -> B {
        assert!(idx < self.len);
        assert!(self.is_bundle_compatible::<B>());
        // swap this and last item
        unsafe { self.buf.swap_columns(idx, self.len - 1) };
        // drop last item
        self.pop()
    }

    pub fn pop<B : DynamicBundle>(&mut self) -> B {
        assert!(self.len > 0);
        assert!(self.is_bundle_compatible::<B>());
        self.len -= 1;
        unsafe { self.take_column_unchecked(self.len) }
    }

    // --- BATCH OPERATIONS --- //
    pub fn extend<I: IntoIterator>(&mut self, iter: I)
    where I::IntoIter: DynamicBundleIter {
        assert!(self.is_bundle_compatible::<I::Item>());
        let iter = iter.into_iter();
        let add_size = match iter.size_hint() {
            (_, Some(upper)) => upper,
            (lower, None) => lower,
        };

        // attempt to pre-reserve the space
        self.reserve(self.len + add_size);

        for item in iter {
            unsafe { self.push_unchecked(item); }
        }
    }

    pub fn extend_with<B: DynamicBundle>(&mut self, n: usize, f: impl FnMut(usize) -> B) {
        self.extend((0..n).map(f))
    }

    pub fn extend_default<B: DynamicBundle + Default>(&mut self, n: usize) {
        self.extend_with(n, |_| B::default())
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        for i in 0..self.len { unsafe { self.buf.drop_column(i) } }
    }
}


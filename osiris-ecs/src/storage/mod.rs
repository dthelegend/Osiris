use crate::storage::raw_table::RawTable;
use crate::storage::type_data::{DynamicBundle, TypeMetadata};

mod type_data;
mod raw_table;
mod test;
mod query;

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

    pub fn new_for_bundle<B: DynamicBundle>() -> Self {
        Self {
            buf: RawTable::new_unchecked(B::type_metadata()),
            len: 0,
        }
    }

    pub fn from_iter<I: IntoIterator<Item: DynamicBundle, IntoIter: ExactSizeIterator<Item: DynamicBundle>>>(iter: I) -> Self {
        let mut init = Self::new_for_bundle::<I::Item>();
        init.extend(iter);
        init
    }

    pub fn from_fn<B: DynamicBundle>(n: usize, f: impl FnMut(usize) -> B) -> Self {
        let mut init = Self::new_for_bundle::<B>();
        init.extend_from_fn(n, f);
        init
    }

    // --- META OPERATIONS --- //
    pub fn len(&self) -> usize { self.len }
    pub fn capacity(&self) -> usize { self.buf.capacity() }
    pub fn empty(&self) -> bool { self.len == 0 }

    pub fn reserve(&mut self, capacity: usize) {
        self.buf.reserve(capacity);
    }

    pub fn clear(&mut self) {
        for idx in 0..std::mem::replace(&mut self.len, 0) {
            unsafe { self.buf.drop_column(idx) }
        }
        unsafe { self.buf.clear(); }
    }

    fn is_bundle_compatible<B: DynamicBundle>(&self) -> bool {
        std::iter::zip(self.buf.type_metadata(), B::type_metadata()).all(|(a, b )| a == b)
    }

    // --- SINGLE OPERATIONS --- //

    // unchecked bundle operation primitive
    unsafe fn put_column_unchecked(&self, idx: usize, data: impl DynamicBundle) {
        unsafe {
            let mut column = self.buf.column_iter(idx);
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
            let mut column = self.buf.column_iter(idx);
            B::take(| dst_ptr, dst_id | {
                let (TypeMetadata { id: src_id, layout, ..} , src_ptr) = column.next().expect("Bundle must have same type ids as table");
                debug_assert_eq!(src_id, dst_id, "Compatible bundles must have types identically ordered");
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, layout.size());
            })
        }
    }

    // unchecked bundle operation primitive
    unsafe fn put_column_from_iter_unchecked(&self, idx: usize, columns: impl IntoIterator<Item: DynamicBundle, IntoIter: ExactSizeIterator<Item: DynamicBundle>>) -> usize {
        let mut count = 0;
        for (data, mut column) in std::iter::zip(columns.into_iter(), self.buf.column_iter_range(idx, self.capacity() - idx)) {
            unsafe {
                data.put(|src_ptr, src_id| {
                    let (TypeMetadata { id: dst_id, layout, .. }, dst_ptr) = column.next().expect("Compatible Bundles must have same number of type ids as table");
                    debug_assert_eq!(src_id, dst_id, "Compatible bundles must have types identically ordered");
                    std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, layout.size());
                });
            }
            count += 1;
        }
        count
    }

    pub fn push<B: DynamicBundle>(&mut self, data: B) {
        assert!(self.is_bundle_compatible::<B>());
        self.reserve(self.len + 1);

        unsafe { self.put_column_unchecked(self.len, data); }

        self.len += 1;
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
    pub fn swap_pop<B: DynamicBundle>(&mut self, idx: usize) -> B {
        assert!(idx < self.len);
        assert!(self.is_bundle_compatible::<B>());
        // swap this and last item
        unsafe { self.buf.swap_columns(idx, self.len - 1) };
        // drop last item
        self.pop()
    }

    // We drop internally
    pub fn swap_remove(&mut self, idx: usize) {
        assert!(idx < self.len);
        self.len -= 1;
        unsafe {
            // swap this and last item
            self.buf.swap_columns(idx, self.len);
            // drop last item
            self.buf.drop_column(self.len);
        }
    }

    pub fn pop<B : DynamicBundle>(&mut self) -> B {
        assert!(self.len > 0);
        assert!(self.is_bundle_compatible::<B>());
        self.len -= 1;
        unsafe { self.take_column_unchecked(self.len) }
    }

    // --- BATCH OPERATIONS --- //
    pub fn extend<I: IntoIterator<Item: DynamicBundle, IntoIter: ExactSizeIterator<Item: DynamicBundle>>>(&mut self, iter: I) {
        assert!(self.is_bundle_compatible::<I::Item>(), "Incompatible bundles used!");
        let mut iter = iter.into_iter();
        let add_size = match iter.size_hint() {
            (_, Some(upper)) => upper,
            (lower, None) => lower,
        };

        // attempt to pre-reserve the space
        self.reserve(self.len + add_size);

        unsafe {
            self.len += self.put_column_from_iter_unchecked(self.len, iter.by_ref().take(add_size));
        }

        for remaining_item in iter {
            self.push(remaining_item);
        }
    }

    pub fn extend_from_fn<B: DynamicBundle>(&mut self, n: usize, f: impl FnMut(usize) -> B) {
        self.extend((0..n).map(f))
    }

    pub fn extend_default<B: DynamicBundle + Default>(&mut self, n: usize) {
        self.extend_from_fn(n, |_| B::default())
    }

    pub fn extend_cloned<B: DynamicBundle + Clone>(&mut self, n: usize, prototype: B) {
        self.extend((0..n).map(|_| prototype.clone()))
    }

    // Add and use query once available
    // pub fn remove_if<B: DynamicBundle>(&self, mut pred: impl FnMut(usize, &B) -> bool) {
    //     todo!()
    // }

    pub fn erase(&self, idx: usize, count: usize) {
        assert!(idx + count < self.len());

        for drop_idx in idx..(idx + count) {
            unsafe { self.buf.drop_column(drop_idx) }
        }

        unsafe { self.buf.move_columns(idx + count, self.len - (idx + count), idx) }
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        for i in 0..self.len { unsafe { self.buf.drop_column(i) } }
    }
}

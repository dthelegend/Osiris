use crate::storage::raw_table::RawTable;
use frunk::hlist::HList;
use frunk::{HCons, HNil};
use std::alloc::GlobalAlloc;
use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::storage::type_data::TypeMetadata;

mod type_data;
mod raw_table;

// We can't quite type mark this because we need to be able to construct a homogenous list of Tables :(
struct Table {
    buf: RawTable,
    len: usize,
}

impl Table {
    fn new(types: impl IntoIterator<Item = TypeMetadata>) -> Self {
        Self {
            buf: RawTable::new(types),
            len: 0,
        }
    }

    fn reserve(&mut self, capacity: usize) {
        self.buf.reserve(capacity);
    }

    // Add an element to the table
    unsafe fn push_back_raw_unchecked<Iterable: Iterator<Item = NonNull<u8>>>(&mut self, data: Iterable)
    where <Iterable as IntoIterator>::IntoIter: ExactSizeIterator {
        self.buf.reserve(self.len + 1);
        unsafe {
            self.buf.write_raw_unchecked(self.len, data);
        }
        self.len += 1;
    }
    
    pub fn pop(&mut self) {
        assert!(self.len > 0);
        self.len -= 1;
        unsafe {
            self.buf.drop_unchecked(self.len);
        }
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        while self.len > 0 { self.pop() }
    }
}


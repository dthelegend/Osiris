use crate::storage::raw_table::RawTable;
use frunk::hlist::HList;
use frunk::{HCons, HNil};
use std::alloc::GlobalAlloc;
use std::marker::PhantomData;
use crate::storage::type_data::{TypeData, TypeMetadata};

mod type_data;
mod raw_table;

// We can't quite type mark this because we need to be able to construct a homogenous list of Tables
struct Table<const N: usize> {
    buf: RawTable<N>,
    len: usize,
}

trait ToMetadataArray<const N: usize> {
    fn metadata() -> [TypeMetadata; N];
}

impl ToMetadataArray<0> for HNil {
    fn metadata() -> [TypeMetadata; 0] { [] }
}

impl <const N: usize, A: 'static + Sized, B: ToMetadataArray<{ N - 1 }>> ToMetadataArray<N> for HCons<A, B> {
    fn metadata() -> [TypeMetadata; N] {
        let mut f = std::iter::once(TypeMetadata::of::<A>()).chain(B::metadata().into_iter());
        std::array::from_fn(|_| f.next().unwrap())
    }
}

impl <const N: usize, A: ToMetadataArray<N>> Table<N> {
    fn new() -> Self {
        Self {
            buf: RawTable::new(A::metadata()),
            len: 0,
        }
    }

    fn reserve(&mut self, capacity: usize) {
        self.buf.reserve(capacity);
    }
}

struct Query<const N: usize> {
    &mut Table
}


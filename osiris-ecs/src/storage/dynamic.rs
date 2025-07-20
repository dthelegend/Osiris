use std::marker::PhantomData;
use frunk::ToMut;
use super::traits::{MutableStorage, ToVec, Zippable};

// Element Definition
pub struct DynamicStorage<T: ToVec> {
    _marker: PhantomData<T>,
    inner: T::AsVec
}

impl <T: ToVec> DynamicStorage<T> {
    pub fn new() -> DynamicStorage<T> {
        Self {
            _marker: PhantomData,
            inner: <T as ToVec>::new()
        }
    }
}

// Storage Definition
impl <'a, T : ToVec> MutableStorage<'a> for DynamicStorage<T>
where <T as ToVec>::AsVec: ToMut<'a, Output: Zippable<Item: 'a>> {
    type Item = <<<T as ToVec>::AsVec as ToMut<'a>>::Output as Zippable>::Item;

    fn iter(&'a mut self) -> impl Iterator<Item=Self::Item> {
        Zippable::zip(self.inner.to_mut())
    }
}

use std::marker::PhantomData;
use frunk::ToMut;
use super::traits::{MutableStorage, ToArray, Zippable};

// Element Definition
#[derive(Debug)]
pub struct StaticStorage<T: ToArray<N>, const N: usize> {
    _marker: PhantomData<T>,
    inner: T::AsArray
}

impl <'a, T : ToArray<N> + Clone, const N: usize> StaticStorage<T, N> {
    pub fn from_element(el: T) -> Self {
        Self {
            _marker: PhantomData,
            inner: <T as ToArray<N>>::from_element(el)
        }
    }
}

impl <'a, T : ToArray<N> + Default + Clone, const N: usize> StaticStorage<T, N> {
    pub fn from_default() -> Self {
        Self {
            _marker: PhantomData,
            inner: <T as ToArray<N>>::from_element(T::default())
        }
    }
}

// Storage Definition
impl <'a, T: ToArray<N>, const N: usize> MutableStorage<'a> for StaticStorage<T, N>
where <T as ToArray<N>>::AsArray: ToMut<'a>,
    <<T as ToArray<N>>::AsArray as ToMut<'a>>::Output: Zippable<Item: 'a>,
{
    type Item = <<<T as ToArray<N>>::AsArray as ToMut<'a>>::Output as Zippable>::Item;

    fn iter(&'a mut self) -> impl Iterator<Item=Self::Item> {
        self.inner.to_mut().zip()
    }
}


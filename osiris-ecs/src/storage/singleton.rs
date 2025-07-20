use frunk::{ToMut};
use super::traits::MutableStorage;

// Element Definition
#[repr(transparent)]
pub struct SingletonStorage<T>(T);

impl <T> From<T> for SingletonStorage<T> {
    fn from(el: T) -> Self{
        Self(el)
    }
}

impl <T: Default> Default for SingletonStorage<T>{
    fn default() -> Self {
        Self(T::default())
    }
}

// Storage Definition
impl <'a, T: ToMut<'a>> MutableStorage<'a> for SingletonStorage<T>
where <T as ToMut<'a>>::Output: 'a {
    type Item = <T as ToMut<'a>>::Output;

    fn iter(&'a mut self) -> impl Iterator<Item=Self::Item> {
        std::iter::once(self.0.to_mut())
    }
}

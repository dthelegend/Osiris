#![feature(iter_array_chunks)]

// Just a tag trait to seal components
trait Component {}

mod storage;

use std::marker::PhantomData;
use frunk::{HList, HNil, ToMut};
use frunk::hlist::{HList, Sculptor};
use storage::*;
use storage::traits::{ToArray, ToVec};
use crate::storage::dynamic::DynamicStorage;
use crate::storage::fixed::StaticStorage;
use crate::storage::singleton::SingletonStorage;
use crate::storage::traits::MutableStorage;

struct ArchetypeBuilder<A: ComponentList> {
    inner: PhantomData<A>
}

impl ArchetypeBuilder<HNil> {
    const fn new() -> Self {
        Self {
            inner: PhantomData
        }
    }
}

impl <A : ComponentList> ArchetypeBuilder<A> {
    const fn add_component<C: Component>(self) -> ArchetypeBuilder<HList!(C, ...A)> {
        ArchetypeBuilder {
            inner: PhantomData
        }
    }
}

impl <A: ComponentList> ArchetypeBuilder<A> {
    fn build_singleton(self, prototype: A) -> Archetype<A, SingletonStorage<A>>
    {
        Archetype {
            _marker: PhantomData,
            storage: SingletonStorage::from(prototype)
        }
    }

    fn build_dynamic(self) -> Archetype<A, DynamicStorage<A>>
    where A: ToVec
    {
        Archetype {
            _marker: PhantomData,
            storage: DynamicStorage::new()
        }
    }
}

impl <A: ComponentList + Clone> ArchetypeBuilder<A> {
    fn build_static<const N: usize>(self, prototype: A) -> Archetype<A, StaticStorage<A, N>>
    where A: ToArray<N> {
        Archetype {
            _marker: PhantomData,
            storage: StaticStorage::from_element(prototype)
        }
    }
}

impl <A: ComponentList + Default + Clone> ArchetypeBuilder<A> {
    fn build_static_default<const N: usize>(self) -> Archetype<A, StaticStorage<A, N>>
    where A: ToArray<N> {
        Archetype {
            _marker: PhantomData,
            storage: StaticStorage::from_default()
        }
    }
}

pub struct Archetype<A, Storage> {
    _marker: PhantomData<A>,
    storage: Storage
}

impl <'a, A, Storage: MutableStorage<'a>> Archetype<A, Storage> {
    fn apply<Indices, B: ToMut<'a>, Operation>(&'a mut self, operation: Operation)
    where
        <Storage as MutableStorage<'a>>::Item: Sculptor<B, Indices>,
        Operation: Fn(B) -> () {
        self.storage.iter().for_each(|x| operation(x.sculpt().0))
    }
}

#[cfg(test)]
mod test;

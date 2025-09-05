use std::any::TypeId;
use std::marker::PhantomData;
use crate::storage::Table;

struct Query<Q> {
    _marker: PhantomData<Q>,
}

struct TypeAccess {
    is_mutable: bool,
    type_id: TypeId
}

impl TypeAccess {
    fn mut_for<A: 'static>() -> Self {
        TypeAccess {
            is_mutable: true,
            type_id: TypeId::of::<A>()
        }
    }

    fn ref_for<A: 'static>() -> Self {
        TypeAccess {
            is_mutable: false,
            type_id: TypeId::of::<A>()
        }
    }
}

struct Accessor<'a> {

}

trait Accessible {
    fn access_for() -> TypeAccess;
}

impl <'a, A: 'static> Accessible for &'a mut A {
    fn access_for() -> TypeAccess { TypeAccess::mut_for::<A>() }
}

impl <'a, A: 'static> Accessible for &'a A {
    fn access_for() -> TypeAccess { TypeAccess::ref_for::<A>() }
}

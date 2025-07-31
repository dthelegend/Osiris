use std::alloc::Layout;
use std::any::TypeId;
use std::ptr::NonNull;
use frunk::labelled::chars::T;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct TypeMetadata {
    pub id: TypeId,
    pub layout: Layout,
    pub drop: unsafe fn(*mut u8),
}

impl TypeMetadata {
    pub const unsafe fn from_raw_parts(id: TypeId, layout: Layout, drop: unsafe fn(*mut u8)) -> Self {
        Self { id, layout, drop }
    }

    pub const fn of<T: 'static + Sized>() -> Self {
        // This is very C++
        unsafe fn drop_ptr<T>(x: *mut u8) {
            x.cast::<T>().drop_in_place()
        }
        
        unsafe { Self::from_raw_parts(TypeId::of::<T>(), Layout::new::<T>(), drop_ptr::<T>) }
    }
}

// A bundle represents something that can be put into a table
pub trait DynamicBundle {
    // iterator of the ids contained within this bundle
    fn ids() -> impl IntoIterator<Item=TypeMetadata>;
    fn put(self, f: impl Fn(*mut u8, TypeId));
}

impl <A: 'static + Sized> DynamicBundle for (A,) {
    fn ids() -> impl IntoIterator<Item=TypeMetadata> {
        [TypeMetadata::of::<A>()]
    }

    fn put(mut self, f: impl Fn(*mut u8, TypeId)) {
        f((&mut self.0 as *mut A).cast::<u8>(), TypeId::of::<T>());
        std::mem::forget(self.0);
    }
}




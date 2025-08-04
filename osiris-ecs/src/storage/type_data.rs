use std::alloc::Layout;
use std::any::TypeId;
use std::cmp::Ordering;
use std::mem::MaybeUninit;

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

impl PartialOrd for TypeMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for TypeMetadata {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

// A bundle represents something that can be put into a table
pub unsafe trait DynamicBundle {
    // iterator of the ids contained within this bundle
    fn type_ids() -> impl IntoIterator<Item=TypeMetadata>;
    unsafe fn put(self, f: impl FnMut(*mut u8, TypeId));
    unsafe fn take(f: impl FnMut(*mut u8, TypeId)) -> Self;
}

unsafe impl <A: 'static + Sized> DynamicBundle for (A,) {
    fn type_ids() -> impl IntoIterator<Item=TypeMetadata> {
        let mut x = [TypeMetadata::of::<A>()];
        x.sort_unstable();
        x
    }

    unsafe fn put(mut self, mut f: impl FnMut(*mut u8, TypeId)) {
        f((&mut self.0 as *mut A).cast::<u8>(), TypeId::of::<A>());
        std::mem::forget(self.0);
    }

    unsafe fn take(mut f: impl FnMut(*mut u8, TypeId)) -> Self {
        let mut raw = MaybeUninit::<A>::uninit();
        f(raw.as_mut_ptr().cast(), TypeId::of::<A>());
        (raw.assume_init(),)
    }
}

// Replace this with a trait alias when possible
pub trait DynamicBundleIter : Iterator<Item: DynamicBundle> {
}

impl <T : Iterator<Item:DynamicBundle>> DynamicBundleIter for T {}

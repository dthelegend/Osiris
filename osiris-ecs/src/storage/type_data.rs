use std::alloc::Layout;
use std::any::TypeId;
use std::cmp::Ordering;
use std::mem::MaybeUninit;
use paste::paste;

#[derive(Copy, Clone, Debug)]
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

impl PartialEq<Self> for TypeMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TypeMetadata {
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
    fn type_metadata() -> impl IntoIterator<Item=TypeMetadata>;
    unsafe fn put(self, f: impl FnMut(*mut u8, TypeId));
    unsafe fn take(f: impl FnMut(*mut u8, TypeId)) -> Self;
}

macro_rules! tuple_bundle_impl {
    ($($tuple_types:ty),*) => {
        paste! {
            unsafe impl <$($tuple_types: 'static + Sized),*> DynamicBundle for ($($tuple_types,)*)
            where ($($tuple_types,)*): Sized {
                fn type_metadata() -> impl IntoIterator<Item=TypeMetadata> {
                    let mut x = [$(TypeMetadata::of::<$tuple_types>()),*];
                    x.sort_unstable();
                    x
                }

                unsafe fn put(mut self, mut f: impl FnMut(*mut u8, TypeId)) {
                    let ($([< raw_ $tuple_types:snake >],)*) = &mut self;
                    let mut x = [$((([< raw_ $tuple_types:snake >] as *mut $tuple_types).cast::<u8>(), TypeId::of::<$tuple_types>())),*];
                    x.sort_unstable_by_key(|(_,id)| *id);
                    for (a, b) in x.into_iter() {
                        f(a, b);
                    }
                    std::mem::forget(self);
                }

                unsafe fn take(mut f: impl FnMut(*mut u8, TypeId)) -> Self {
                    let mut raw = ($(MaybeUninit::<$tuple_types>::uninit(),)*);
                    let ($([< raw_ $tuple_types:snake >],)*) = &mut raw;
                    let mut refs = [
                        $(
                        {
                            ([< raw_ $tuple_types:snake >].as_mut_ptr().cast(), TypeId::of::<$tuple_types>())
                        }
                        ),*
                    ];
                    refs.sort_unstable_by_key(|(_,id)| *id);
                    for (a, b) in refs.into_iter() {
                        f(a, b);
                    }
                    // NB: Since raw is maybe uninit we don't need to drop it
                    unsafe { std::mem::transmute_copy::<_,Self>(&raw) }
                }
            }
        }
    };
}

macro_rules! all_tuple_impl_for {
    ($single:ty) => {
        tuple_bundle_impl!($single);
    };
    ($single:ty, $($list:ty),+) => {
        tuple_bundle_impl!($single, $($list),+);
        all_tuple_impl_for!($($list),+);
    };
}

all_tuple_impl_for!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

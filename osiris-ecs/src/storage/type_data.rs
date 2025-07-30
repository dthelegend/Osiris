use std::alloc::Layout;
use std::any::TypeId;
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


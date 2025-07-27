use std::alloc::Layout;
use std::any::TypeId;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub struct TypeMetadata {
    pub id: TypeId,
    pub layout: Layout
}

impl TypeMetadata {
    pub const unsafe fn from_raw_parts(id: TypeId, layout: Layout) -> Self {
        Self { id, layout }
    }

    pub const fn of<T: 'static + Sized>() -> Self {
        unsafe { Self::from_raw_parts(TypeId::of::<T>(), Layout::new::<T>()) }
    }
}

pub struct TypeData {
    pub metadata: TypeMetadata,
    pub ptr: NonNull<u8>,
}

#[repr(transparent)]
pub struct SortedTypeDataArray<const N: usize>([TypeData; N]);

impl <const N: usize> Deref for SortedTypeDataArray<N> {
    type Target = [TypeData; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl <const N: usize> DerefMut for SortedTypeDataArray<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> SortedTypeDataArray<N> {
    pub fn new(mut type_metadata: [TypeMetadata; N]) -> Self {
        type_metadata.sort_unstable_by_key(|x| x.id);
        unsafe { Self::new_unchecked(type_metadata) }
    }

    pub unsafe fn new_unchecked(type_metadata: [TypeMetadata; N]) -> Self {
        Self (type_metadata.map(|metadata| TypeData { ptr: metadata.layout.dangling(), metadata }))
    }

    pub fn search_dynamic(&self, type_id: TypeId) -> Option<&TypeData> {
        self.binary_search_by_key(&type_id, |type_data: &TypeData| type_data.metadata.id).ok().map(|idx| &self[idx])
    }

    pub fn search<T: 'static>(&self) -> Option<&TypeData> {
        self.search_dynamic(TypeId::of::<T>())
    }
}

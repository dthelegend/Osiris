use frunk::{hlist, HCons, HList, HNil};

// Generic
pub trait MutableStorage<'a> {
    type Item: 'a;
    fn iter(&'a mut self) -> impl Iterator<Item = Self::Item>;
}

// Zippable
pub trait Zippable {
    type Item;
    fn zip(self) -> impl Iterator<Item=Self::Item>;
}

impl Zippable for HNil {
    type Item = HNil;
    fn zip(self) -> impl Iterator<Item=Self::Item> {
        std::iter::repeat(HNil)
    }
}

impl<T: IntoIterator, TS: Zippable> Zippable for HCons<T, TS>
{
    type Item = HList![<T as IntoIterator>::Item, ...<TS as Zippable>::Item];

    fn zip(self) -> impl Iterator<Item=Self::Item> {
        self.head.into_iter().zip(self.tail.zip()).map(|(l, r)| hlist![l, ...r])
    }
}

// Vector Conversion helpers
pub trait VecList {}
impl <T, TS: VecList> VecList for HCons<Vec<T>, TS> {}
impl VecList for HNil {}

pub trait ToVec {
    type AsVec;
    fn new() -> Self::AsVec;
}
impl<A, B: ToVec> ToVec for HCons<A, B> {
    type AsVec = HCons<Vec<A>, B::AsVec>;

    fn new() -> Self::AsVec {
        HCons {
            head: Vec::new(),
            tail: <B as ToVec>::new()
        }
    }
}
impl ToVec for HNil {
    type AsVec = HNil;
    fn new() -> Self::AsVec {
        HNil
    }
}

// Array Conversion Helpers
trait ArrayList<const N: usize> {}
impl <const N: usize, T, TS: ArrayList<N>> ArrayList<N> for HCons<[T; N], TS> {}
impl <const N: usize> ArrayList<N> for HNil {}

pub trait ToArray<const N: usize> {
    type AsArray : ArrayList<N>;
    fn from_element(prototype: Self) -> Self::AsArray;
}
impl<const N: usize, A: Clone, B: ToArray<N>> ToArray<N> for HCons<A, B> {
    type AsArray = HCons<[A; N], B::AsArray>;

    fn from_element(prototype: Self) -> Self::AsArray {
        HCons {
            head: std::array::from_fn(|_| prototype.head.clone()),
            tail: ToArray::<N>::from_element(prototype.tail)
        }
    }
}
impl<const N: usize> ToArray<N> for HNil {
    type AsArray = HNil;
    fn from_element(_: Self) -> Self::AsArray {
        HNil
    }
}
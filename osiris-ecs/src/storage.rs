use super::Component;
use frunk::{HCons, HNil, ToMut};

pub mod singleton;
pub mod dynamic;
pub mod fixed;

pub trait ComponentList {}
impl<A: Component, B: ComponentList> ComponentList for HCons<A, B> {}
impl ComponentList for HNil {}

pub mod traits;


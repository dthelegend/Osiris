use frunk::hlist_pat;
use super::*;

#[derive(Default, Copy, Clone, Debug)]
struct Position(u32, u32, u32);
impl Component for Position {}
#[derive(Default, Copy, Clone, Debug)]
struct Rotation(u32, u32, u32, u32);
impl Component for Rotation {}
#[derive(Default, Copy, Clone, Debug)]
struct Velocity(u32, u32, u32);
impl Component for Velocity {}

#[test]
fn it_works() {
    let mut archetype_static: Archetype<_, _> = ArchetypeBuilder::new()
        .add_component::<Velocity>()
        .add_component::<Rotation>()
        .add_component::<Position>()
        .build_static_default::<3>();

    let mut archetype_dynamic: Archetype<_, _> = ArchetypeBuilder::new()
        .add_component::<Velocity>()
        .add_component::<Rotation>()
        .add_component::<Position>()
        .build_static_default::<3>();

    let mut archetype_singleton: Archetype<_, _> = ArchetypeBuilder::new()
        .add_component::<Velocity>()
        .add_component::<Rotation>()
        .add_component::<Position>()
        .build_static_default::<3>();

    archetype_static.apply(| hlist_pat![pos, vel]: HList!(&'_ mut Position, &'_ mut Velocity) | {
        pos.0 += vel.0;
        pos.1 += vel.1;
        pos.2 += vel.2;
    });

    archetype_dynamic.apply(| hlist_pat![pos, vel]: HList!(&'_ mut Position, &'_ mut Velocity) | {
        pos.0 += vel.0;
        pos.1 += vel.1;
        pos.2 += vel.2;
    });

    archetype_singleton.apply(| hlist_pat![pos, vel]: HList!(&'_ mut Position, &'_ mut Velocity) | {
        pos.0 += vel.0;
        pos.1 += vel.1;
        pos.2 += vel.2;
    });
}

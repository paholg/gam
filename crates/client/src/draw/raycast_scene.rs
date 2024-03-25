use bevy::prelude::{Added, Commands, Component, Entity, Handle, Mesh, Parent, Query, Scene, With};
use bevy_mod_raycast::prelude::RaycastMesh;

/// A Component to be added to entities with `Scene`s, that we want to act as
/// `RaycastMesh<()>`s.
#[derive(Component, Default)]
pub struct RaycastScene;

pub fn raycast_scene_system(
    mut commands: Commands,
    q_meshes: Query<(Entity, &Parent), Added<Handle<Mesh>>>,
    q_children: Query<&Parent>,
    q_parents: Query<&Handle<Scene>, With<RaycastScene>>,
) {
    // A `Scene` has meshes as its grandchildren, so we need a silly bit of
    // indirection to tell if we should add our `RaycastMesh`.
    for (entity, parent) in q_meshes.iter() {
        // FIXME: These queries are the same?????????
        let Ok(parent) = q_children.get(parent.get()) else {
            continue;
        };
        let Ok(grandparent) = q_children.get(parent.get()) else {
            continue;
        };
        if q_parents.get(grandparent.get()).is_ok() {
            commands.entity(entity).insert(RaycastMesh::<()>::default());
        }
    }
}

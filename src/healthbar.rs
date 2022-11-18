use bevy::prelude::{
    Bundle, Component, ComputedVisibility, GlobalTransform, Handle, Mesh, Parent, Query,
    StandardMaterial, Transform, Vec3, Visibility, With, Without,
};
use tracing::info;

#[derive(Component)]
pub struct Healthbar {
    pub displacement: Vec3,
}

#[derive(Bundle)]
pub struct HealthbarBundle {
    pub bar: Healthbar,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

pub fn healthbar_system(
    mut q_healthbar: Query<(&Parent, &mut Transform, &Healthbar)>,
    q_parent: Query<&Transform, Without<Healthbar>>,
) {
    for (parent, mut transform, healthbar) in q_healthbar.iter_mut() {
        let parent_transform = q_parent.get(parent.get()).unwrap();
        let rotation = parent_transform.rotation.inverse();
        transform.rotation = rotation;
        transform.translation = rotation * healthbar.displacement;
    }
}

use bevy::prelude::{
    Bundle, Component, ComputedVisibility, GlobalTransform, Handle, Mesh, Parent, Query,
    StandardMaterial, Transform, Vec3, Visibility, With, Without,
};
use tracing::info;

#[derive(Component)]
pub struct HealthbarMarker;

#[derive(Bundle)]
pub struct Healthbar {
    pub marker: HealthbarMarker,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

pub fn healthbar_system(
    mut q_healthbar: Query<(&Parent, &mut Transform), With<HealthbarMarker>>,
    q_parent: Query<(&Transform), Without<HealthbarMarker>>,
) {
    for (parent, mut transform) in q_healthbar.iter_mut() {
        let parent_transform = q_parent.get(parent.get()).unwrap();
        transform.rotation = parent_transform.rotation.inverse();
        // transform.translation = Vec3::ZERO;
        info!(?transform, ?parent_transform);
    }
}

use bevy::prelude::{
    default, shape, Added, Assets, BuildChildren, Bundle, Children, Color, Commands, Component,
    ComputedVisibility, CoreStage, Entity, GlobalTransform, Mesh, Parent, PbrBundle,
    Plugin, Query, ResMut, StandardMaterial, Transform, Vec2, Vec3, Visibility, With, Without,
};
use tracing::{warn};

use crate::Health;

#[derive(Component)]
pub struct Healthbar {
    displacement: Vec3,
    size: Vec2,
}

impl Default for Healthbar {
    fn default() -> Self {
        Self {
            displacement: Vec3::new(0.0, -0.7, 0.01),
            size: Vec2::new(1.8, 0.3),
        }
    }
}

#[derive(Component)]
struct HealthbarMarker;

#[derive(Bundle)]
struct HealthbarBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    marker: HealthbarMarker,
}

#[derive(Component)]
struct BarMarker;

fn add_healthbar_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    healthbars: Query<(Entity, &Healthbar), Added<Healthbar>>,
) {
    for (parent, healthbar) in healthbars.iter() {
        let bar = commands
            .spawn((
                PbrBundle {
                    material: materials.add(Color::GREEN.into()),
                    mesh: meshes.add(
                        shape::Quad {
                            size: healthbar.size,
                            ..default()
                        }
                        .into(),
                    ),
                    transform: Transform::from_translation(healthbar.displacement),
                    ..default()
                },
                BarMarker,
            ))
            .id();
        let background = commands
            .spawn(PbrBundle {
                material: materials.add(Color::BLACK.into()),
                mesh: meshes.add(
                    shape::Quad {
                        size: healthbar.size,
                        ..default()
                    }
                    .into(),
                ),
                transform: Transform::from_translation(
                    healthbar.displacement - Vec3::new(0.0, 0.0, 0.01),
                ),
                ..default()
            })
            .id();
        let bundle = commands
            .spawn(HealthbarBundle {
                marker: HealthbarMarker,
                transform: Transform::default(),
                global_transform: GlobalTransform::default(),
                visibility: Visibility::VISIBLE,
                computed_visibility: ComputedVisibility::default(),
            })
            .id();
        commands.entity(bundle).push_children(&[bar, background]);
        commands.entity(parent).push_children(&[bundle]);
    }
}

fn healthbar_update_system(
    mut q_healthbar: Query<(&Parent, &Children, &mut Transform), With<HealthbarMarker>>,
    q_parent: Query<(&Transform, &Health, &Healthbar), Without<HealthbarMarker>>,
    mut q_child: Query<
        &mut Transform,
        (With<BarMarker>, Without<HealthbarMarker>, Without<Health>),
    >,
) {
    for (parent, children, mut transform) in q_healthbar.iter_mut() {
        let (parent_transform, health, healthbar) = q_parent.get(parent.get()).unwrap();
        let healthiness = health.cur / health.max;
        let rotation = parent_transform.rotation.inverse();
        transform.rotation = rotation;
        transform.translation = rotation * healthbar.displacement;

        // The bar is the first child, so we don't need to iterate over all of them.
        if let Some(&child) = children.iter().next() {
            if let Ok(mut bar_transform) = q_child.get_mut(child) {
                bar_transform.scale = Vec3::new(healthiness, 1.0, 1.0);
                let offset = healthbar.size.x * 0.5 * (1.0 - healthiness);
                bar_transform.translation = healthbar.displacement - Vec3::new(offset, 0.0, 0.0);
            } else {
                warn!("HealthbarMarker's first child is incorrect!");
            }
        } else {
            warn!("HealthbarMarker does not have a child");
        }
    }
}

pub struct HealthbarPlugin;

impl Plugin for HealthbarPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // We need to fix the position after bevy changes it.
        app.add_system_to_stage(CoreStage::PostUpdate, healthbar_update_system)
            .add_system(add_healthbar_system);
    }
}

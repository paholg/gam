use std::marker::PhantomData;

use bevy::app::Plugin;
use bevy::app::Startup;
use bevy::app::Update;
use bevy::asset::Assets;
use bevy::asset::Handle;
use bevy::color::LinearRgba;
use bevy::math::Vec3;
use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::pbr::PbrBundle;
use bevy::pbr::StandardMaterial;
use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Mesh;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::prelude::Sphere;
use bevy::prelude::Transform;
use bevy::prelude::Without;
use bevy::prelude::World;
use bevy_rapier3d::prelude::Velocity;
use engine::ability::grenade::FragGrenade;
use engine::ability::grenade::Grenade;
use engine::ability::grenade::HealGrenade;
use engine::FootOffset;

use super::HasOutline;
use crate::color_gradient::ColorGradient;
use crate::draw::explosion::ExplosionAssets;
use crate::in_plane;
use crate::shapes::HollowPolygon;

pub struct GrenadePlugin;

#[derive(Resource)]
pub struct GrenadeAssets<G: Grenade> {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    outline_mesh: Handle<Mesh>,
    outline_material: Handle<StandardMaterial>,
    pub explosion: ExplosionAssets,
    _marker: PhantomData<G>,
}

impl Plugin for GrenadePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                draw_grenade::<FragGrenade>,
                draw_grenade::<HealGrenade>,
                draw_grenade_outline::<FragGrenade>,
                draw_grenade_outline::<HealGrenade>,
            ),
        );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Frag
    let frag_grenade_material = StandardMaterial {
        emissive: LinearRgba::rgb(10.0, 0.0, 1.0),
        ..Default::default()
    };

    let frag_assets = GrenadeAssets::<FragGrenade> {
        mesh: meshes.add(Sphere::new(1.0)),
        material: materials.add(frag_grenade_material.clone()),
        outline_mesh: meshes.add(HollowPolygon {
            radius: 1.0,
            thickness: 0.06,
            vertices: 60,
        }),
        outline_material: materials.add(frag_grenade_material),
        explosion: ExplosionAssets::new(
            ColorGradient::new([
                (0.0, LinearRgba::new(50.0, 12.0, 0.0, 0.2)),
                (0.5, LinearRgba::new(100.0, 25.0, 0.0, 0.2)),
                (0.8, LinearRgba::new(2.0, 2.0, 2.0, 0.2)),
                (1.0, LinearRgba::new(0.0, 0.0, 0.0, 0.1)),
            ]),
            &mut meshes,
            &mut materials,
        ),
        _marker: PhantomData,
    };
    let heal_grenade_material = StandardMaterial {
        emissive: LinearRgba::rgb(0.0, 10.0, 1.0),
        ..Default::default()
    };

    let heal_assets = GrenadeAssets::<HealGrenade> {
        mesh: meshes.add(Sphere::new(1.0)),
        outline_mesh: meshes.add(HollowPolygon {
            radius: 1.0,
            thickness: 0.06,
            vertices: 60,
        }),
        material: materials.add(heal_grenade_material.clone()),
        outline_material: materials.add(heal_grenade_material),
        explosion: ExplosionAssets::new(
            ColorGradient::new([
                (0.0, LinearRgba::new(0.0, 50.0, 0.0, 0.2)),
                (0.5, LinearRgba::new(0.0, 100.0, 0.0, 0.2)),
                (0.8, LinearRgba::new(2.0, 2.0, 2.0, 0.2)),
                (1.0, LinearRgba::new(0.0, 0.0, 0.0, 0.1)),
            ]),
            &mut meshes,
            &mut materials,
        ),
        _marker: PhantomData,
    };

    commands.add(|world: &mut World| {
        world.insert_resource(frag_assets);
        world.insert_resource(heal_assets);
    });
}

pub fn draw_grenade<G: Grenade>(
    mut commands: Commands,
    assets: Res<GrenadeAssets<G>>,
    query: Query<Entity, Added<G>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert(InheritedVisibility::default());
        ecmds.with_children(|builder| {
            builder.spawn(PbrBundle {
                material: assets.material.clone_weak(),
                mesh: assets.mesh.clone_weak(),
                ..Default::default()
            });
        });
    }
}

pub fn draw_grenade_outline<G: Grenade>(
    mut commands: Commands,
    assets: Res<GrenadeAssets<G>>,
    query: Query<(Entity, &Transform, &Velocity, &FootOffset, &G), Without<HasOutline>>,
) {
    for (entity, transform, velocity, foot_offset, grenade) in &query {
        // TODO: This should be a better query. Maybe just a timeout? What if it never
        // stops moving?
        if velocity.linvel.length_squared() < 0.1 * 0.1 {
            commands
                .entity(entity)
                .insert(HasOutline)
                .with_children(|builder| {
                    builder.spawn((
                        PbrBundle {
                            mesh: assets.outline_mesh.clone_weak(),
                            material: assets.outline_material.clone_weak(),
                            transform: in_plane()
                                .with_translation(Vec3::new(0.0, foot_offset.y + 0.01, 0.0))
                                .with_scale(transform.scale.recip() * grenade.explosion_radius()),
                            ..Default::default()
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                });
        }
    }
}

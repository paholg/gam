use std::marker::PhantomData;

use bevy::{
    app::{Plugin, Startup, Update},
    asset::Assets,
    ecs::{
        component::Component,
        entity::Entity,
        query::{Added, Without},
        system::{Commands, Query, Res, ResMut, Resource},
        world::World,
    },
    hierarchy::BuildChildren,
    math::{primitives::Sphere, Vec3},
    pbr::{NotShadowCaster, NotShadowReceiver, PbrBundle},
    prelude::{Color, Handle, Mesh, StandardMaterial},
};
use bevy_rapier3d::dynamics::Velocity;
use engine::{
    ability::grenade::{Frag, Grenade, GrenadeKind, GrenadeProps, Heal},
    FootOffset,
};

use crate::{
    asset_handler::explosion::ExplosionAssets, color_gradient::ColorGradient, draw::ObjectGraphics,
    in_plane, shapes::HollowPolygon,
};

pub struct GrenadePlugin;

impl Plugin for GrenadePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (draw_grenade_system, draw_grenade_outline_system));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    frag_props: Res<GrenadeProps<Frag>>,
    heal_props: Res<GrenadeProps<Heal>>,
) {
    // Frag grenade
    let frag_grenade_material = StandardMaterial {
        emissive: Color::rgb_linear(10_000.0, 0.0, 100.0),
        ..Default::default()
    };

    let frag = GrenadeAssets::<Frag> {
        mesh: meshes.add(Sphere::new(frag_props.radius)),
        material: materials.add(frag_grenade_material.clone()),
        outline_mesh: meshes.add(HollowPolygon {
            radius: frag_props.explosion.max_radius,
            thickness: 0.06,
            vertices: 60,
        }),
        outline_material: materials.add(frag_grenade_material),
        explosion: ExplosionAssets::new(
            &mut meshes,
            &mut materials,
            ColorGradient::new([
                (0.0, Color::rgba(50.0, 12.0, 0.0, 0.2)),
                (0.5, Color::rgba(100.0, 25.0, 0.0, 0.2)),
                (0.8, Color::rgba(2.0, 2.0, 2.0, 0.2)),
                (1.0, Color::rgba(0.0, 0.0, 0.0, 0.1)),
            ]),
        ),
        _marker: PhantomData,
    };

    // Heal grenade
    let heal_grenade_material = StandardMaterial {
        emissive: Color::rgb_linear(0.0, 10_000.0, 100.0),
        ..Default::default()
    };

    let heal = GrenadeAssets::<Heal> {
        mesh: meshes.add(Sphere::new(heal_props.radius)),
        outline_mesh: meshes.add(HollowPolygon {
            radius: heal_props.explosion.max_radius,
            thickness: 0.06,
            vertices: 60,
        }),
        material: materials.add(heal_grenade_material.clone()),
        outline_material: materials.add(heal_grenade_material),
        explosion: ExplosionAssets::new(
            &mut meshes,
            &mut materials,
            ColorGradient::new([
                (0.0, Color::rgba(0.0, 50.0, 0.0, 0.2)),
                (0.5, Color::rgba(0.0, 100.0, 0.0, 0.2)),
                (0.8, Color::rgba(2.0, 2.0, 2.0, 0.2)),
                (1.0, Color::rgba(0.0, 0.0, 0.0, 0.1)),
            ]),
        ),
        _marker: PhantomData,
    };

    // Finish
    commands.add(|world: &mut World| {
        world.insert_resource(frag);
        world.insert_resource(heal);
    });
}

#[derive(Resource)]
struct GrenadeAssets<G> {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    outline_mesh: Handle<Mesh>,
    outline_material: Handle<StandardMaterial>,
    explosion: ExplosionAssets,
    _marker: PhantomData<G>,
}

fn draw_grenade_system(
    mut commands: Commands,
    frag_assets: Res<GrenadeAssets<Frag>>,
    heal_assets: Res<GrenadeAssets<Heal>>,
    query: Query<(Entity, &Grenade), Added<Grenade>>,
) {
    for (entity, grenade) in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        let (mesh, material) = match grenade.kind {
            GrenadeKind::Frag => (frag_assets.mesh.clone(), frag_assets.material.clone()),
            GrenadeKind::Heal => (heal_assets.mesh.clone(), heal_assets.material.clone()),
        };
        ecmds.insert(ObjectGraphics {
            material,
            mesh,
            ..Default::default()
        });
    }
}

#[derive(Component)]
struct HasOutline;

fn draw_grenade_outline_system(
    mut commands: Commands,
    frag_assets: Res<GrenadeAssets<Frag>>,
    heal_assets: Res<GrenadeAssets<Heal>>,
    query: Query<(Entity, &Grenade, &Velocity, &FootOffset), Without<HasOutline>>,
) {
    for (entity, grenade, velocity, foot_offset) in &query {
        if velocity.linvel.length_squared() < 0.1 * 0.1 {
            let (mesh, material) = match grenade.kind {
                GrenadeKind::Frag => (
                    frag_assets.outline_mesh.clone(),
                    frag_assets.outline_material.clone(),
                ),
                GrenadeKind::Heal => (
                    heal_assets.outline_mesh.clone(),
                    heal_assets.outline_material.clone(),
                ),
            };
            let outline_entity = commands
                .spawn((
                    PbrBundle {
                        mesh,
                        material,
                        transform: in_plane().with_translation(Vec3::new(
                            0.0,
                            foot_offset.y + 0.01,
                            0.0,
                        )),
                        ..Default::default()
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ))
                .id();
            commands
                .entity(entity)
                .insert(HasOutline)
                .push_children(&[outline_entity]);
        }
    }
}

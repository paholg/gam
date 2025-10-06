use bevy::{
    asset::Handle,
    color::{Color, LinearRgba},
    ecs::{hierarchy::ChildOf, resource::Resource},
    light::{NotShadowCaster, NotShadowReceiver},
    math::primitives::Sphere,
    mesh::Mesh,
    pbr::MeshMaterial3d,
    prelude::{
        Added, Assets, Commands, Component, Entity, Mesh3d, Query, Res, ResMut, StandardMaterial,
        Transform, Vec3, With,
    },
    render::alpha::AlphaMode,
};
use engine::{status_effect::Temperature, FootOffset, PLAYER_HEIGHT, PLAYER_R};

use crate::{color_gradient::ColorGradient, draw::character::CharacterMarker};

#[derive(Resource)]
pub struct TemperatureAssets {
    gradient: ColorGradient,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

impl TemperatureAssets {
    fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let gradient = ColorGradient::new([
            (0.0, LinearRgba::new(0.0, 20.0, 50.0, 0.3)),
            (0.5, LinearRgba::new(0.0, 0.0, 0.0, 0.0)),
            (1.0, LinearRgba::new(50.0, 20.0, 0.0, 0.3)),
        ]);
        let mesh = meshes.add(Sphere::new(1.0));
        let material = materials.add(StandardMaterial {
            base_color: Color::NONE,
            emissive: LinearRgba::NONE,
            alpha_mode: AlphaMode::Add,
            unlit: true,
            ..Default::default()
        });

        Self {
            gradient,
            mesh,
            material,
        }
    }
}

pub fn create_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(TemperatureAssets::new(&mut meshes, &mut materials));
}

#[derive(Component)]
pub struct TemperatureGlow;

pub fn draw_temperature_system(
    mut commands: Commands,
    assets: Res<TemperatureAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &FootOffset), Added<CharacterMarker>>,
) {
    for (entity, foot_offset) in query.iter() {
        commands.entity(entity).with_children(|builder| {
            // Clone material because we'll mutate it.
            let material = materials.get(&assets.material).unwrap().clone();
            builder.spawn((
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(materials.add(material)),
                Transform::from_translation(
                    foot_offset.to_vec() + Vec3::new(0.0, PLAYER_HEIGHT * 0.5, 0.0),
                )
                .with_scale(Vec3::new(
                    PLAYER_R * 1.4,
                    PLAYER_HEIGHT * 0.7,
                    PLAYER_R * 1.4,
                )),
                NotShadowCaster,
                NotShadowReceiver,
                TemperatureGlow,
            ));
        });
    }
}

pub fn update_temperature_system(
    assets: Res<TemperatureAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&ChildOf, &MeshMaterial3d<StandardMaterial>), With<TemperatureGlow>>,
    parent_q: Query<&Temperature>,
) {
    for (child_of, material) in &query {
        let Ok(temperature) = parent_q.get(child_of.parent()) else {
            tracing::warn!("TemperatureGlow missing parent");
            continue;
        };

        // Temperature can be anything, we need to map it to [0, 1] for our
        // gradient.
        let gradient_val = (temperature.temp * 0.03).tanh() * 0.5 + 0.5;

        let color = assets.gradient.get(gradient_val);

        let mat = materials.get_mut(material).unwrap();
        mat.emissive = color;
        mat.base_color = color.into();
    }
}

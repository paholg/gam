use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::{
        Added, Assets, BuildChildren, Commands, Component, Entity, Handle, Parent, PbrBundle,
        Query, Res, ResMut, StandardMaterial, Transform, Vec3, With,
    },
};
use engine::{status_effect::Temperature, CharacterMarker, FootOffset, PLAYER_HEIGHT, PLAYER_R};

use crate::asset_handler::AssetHandler;

#[derive(Component)]
pub struct TemperatureGlow;

pub fn draw_temperature_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &FootOffset), Added<CharacterMarker>>,
) {
    for (entity, foot_offset) in query.iter() {
        commands.entity(entity).with_children(|builder| {
            // Clone material because we'll mutate it.
            let material = materials.get(&assets.temperature.material).unwrap().clone();
            builder.spawn((
                PbrBundle {
                    mesh: assets.temperature.mesh.clone(),
                    material: materials.add(material),
                    transform: Transform::from_translation(
                        foot_offset.to_vec() + Vec3::new(0.0, PLAYER_HEIGHT * 0.5, 0.0),
                    )
                    .with_scale(Vec3::new(
                        PLAYER_R * 1.4,
                        PLAYER_HEIGHT * 0.7,
                        PLAYER_R * 1.4,
                    )),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
                TemperatureGlow,
            ));
        });
    }
}

pub fn update_temperature_system(
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Parent, &Handle<StandardMaterial>), With<TemperatureGlow>>,
    parent_q: Query<&Temperature>,
) {
    for (parent, material) in &query {
        let Ok(temperature) = parent_q.get(parent.get()) else {
            tracing::warn!("TemperatureGlow missing parent");
            continue;
        };

        // Temperature can be anything, we need to map it to [0, 1] for our
        // gradient.
        let gradient_val = (temperature.temp * 0.03).tanh() * 0.5 + 0.5;

        let color = assets.temperature.gradient.get(gradient_val);

        let mat = materials.get_mut(material).unwrap();
        mat.emissive = color;
        mat.base_color = color;
    }
}

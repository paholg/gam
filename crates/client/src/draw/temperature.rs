use bevy::pbr::MeshMaterial3d;
use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::Added;
use bevy::prelude::Assets;
use bevy::prelude::BuildChildren;
use bevy::prelude::ChildBuild;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Mesh3d;
use bevy::prelude::Parent;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use bevy::prelude::With;
use engine::status_effect::Temperature;
use engine::CharacterMarker;
use engine::FootOffset;
use engine::PLAYER_HEIGHT;
use engine::PLAYER_R;

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
                Mesh3d(assets.temperature.mesh.clone_weak()),
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
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Parent, &MeshMaterial3d<StandardMaterial>), With<TemperatureGlow>>,
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
        mat.base_color = color.into();
    }
}

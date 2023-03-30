use bevy::{
    prelude::{
        default,
        shape::{self, Icosphere},
        AssetServer, Assets, Color, Commands, Handle, Mesh, ResMut, Resource, StandardMaterial,
    },
    scene::Scene,
};

use crate::{ability::SHOT_R, healthbar::Healthbar};

pub struct HealthbarAssets {
    pub mesh: Handle<Mesh>,
    pub fg_material: Handle<StandardMaterial>,
    pub bg_material: Handle<StandardMaterial>,
}

pub struct ShotAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub struct CharacterAssets {
    pub scene: Handle<Scene>,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
}

// A collection of HandleIds for assets for spawning.
#[derive(Resource)]
pub struct AssetHandler {
    pub healthbar: HealthbarAssets,
    pub shot: ShotAssets,
    pub player: CharacterAssets,
    pub ally: CharacterAssets,
    pub enemy: CharacterAssets,
}

pub fn asset_handler_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let healthbar = HealthbarAssets {
        mesh: meshes.add(
            shape::Quad {
                size: Healthbar::default().size,
                ..default()
            }
            .into(),
        ),
        fg_material: materials.add(Color::GREEN.into()),
        bg_material: materials.add(Color::BLACK.into()),
    };

    let shot = ShotAssets {
        mesh: meshes.add(
            Mesh::try_from(Icosphere {
                radius: SHOT_R,
                subdivisions: 5,
            })
            .unwrap(),
        ),
        material: materials.add(Color::BLUE.into()),
    };

    let spaceship = asset_server.load("models/temp/craft_speederB.glb#Scene0");

    let player = CharacterAssets {
        scene: spaceship.clone(),
        outline_mesh: meshes.add(
            shape::Circle {
                radius: 1.0,
                vertices: 100,
            }
            .into(),
        ),
        outline_material: materials.add(Color::GREEN.into()),
    };

    let ally = CharacterAssets {
        scene: spaceship.clone(),
        outline_mesh: meshes.add(
            shape::Circle {
                radius: 1.0,
                vertices: 100,
            }
            .into(),
        ),
        outline_material: materials.add(Color::CYAN.into()),
    };

    let enemy = CharacterAssets {
        scene: spaceship,
        outline_mesh: meshes.add(
            shape::Circle {
                radius: 1.0,
                vertices: 100,
            }
            .into(),
        ),
        outline_material: materials.add(Color::RED.into()),
    };

    let asset_handler = AssetHandler {
        healthbar,
        shot,
        player,
        ally,
        enemy,
    };
    commands.insert_resource(asset_handler);
}

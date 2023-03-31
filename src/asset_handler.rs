use bevy::{
    prelude::{
        default,
        shape::{self, Icosphere},
        AssetServer, Assets, Color, Commands, Component, Entity, Handle, Mesh, ResMut, Resource,
        StandardMaterial, Vec2, Vec3, Vec4,
    },
    scene::Scene,
};
use bevy_hanabi::{
    ColorOverLifetimeModifier, EffectAsset, Gradient, InitAgeModifier, InitLifetimeModifier,
    InitPositionCircleModifier, InitPositionSphereModifier, InitVelocityCircleModifier,
    InitVelocitySphereModifier, LinearDragModifier, ParticleEffectBundle, ShapeDimension,
    SizeOverLifetimeModifier, Spawner, Value,
};

use crate::{ability::SHOT_R, healthbar::Healthbar, PLAYER_R};

pub struct HealthbarAssets {
    pub mesh: Handle<Mesh>,
    pub fg_material: Handle<StandardMaterial>,
    pub bg_material: Handle<StandardMaterial>,
}

pub struct ShotAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub effect_entity: Entity,
}

pub struct HyperSprintAssets {
    pub effect_entity: Entity,
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
    pub hyper_sprint: HyperSprintAssets,
    pub player: CharacterAssets,
    pub ally: CharacterAssets,
    pub enemy: CharacterAssets,
}

#[derive(Component)]
pub struct ShotEffect;

#[derive(Component)]
pub struct HyperSprintEffect;

pub fn asset_handler_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effects: ResMut<Assets<EffectAsset>>,
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

    let effect = effects.add(shot_effect());
    let effect_entity = commands
        .spawn(ParticleEffectBundle::new(effect))
        .insert(ShotEffect)
        .id();

    let shot = ShotAssets {
        mesh: meshes.add(
            Mesh::try_from(Icosphere {
                radius: SHOT_R,
                subdivisions: 5,
            })
            .unwrap(),
        ),
        material: materials.add(Color::BLUE.into()),
        effect_entity,
    };

    let effect = effects.add(hyper_sprint_effect());
    let effect_entity = commands
        .spawn(ParticleEffectBundle::new(effect))
        .insert(HyperSprintEffect)
        .id();

    let hyper_sprint = HyperSprintAssets { effect_entity };

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
        hyper_sprint,
        player,
        ally,
        enemy,
    };
    commands.insert_resource(asset_handler);
}

fn shot_effect() -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1));
    size_gradient1.add_key(0.3, Vec2::splat(0.1));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let spawner = Spawner::once(250.0.into(), false);

    EffectAsset {
        name: "ShotParticleEffect".to_string(),
        capacity: 32768,
        spawner,
        ..Default::default()
    }
    .init(InitPositionSphereModifier {
        center: Vec3::ZERO,
        radius: SHOT_R,
        dimension: ShapeDimension::Volume,
    })
    .init(InitVelocitySphereModifier {
        center: Vec3::ZERO,
        // Give a bit of variation by randomizing the initial speed
        speed: Value::Uniform((6., 7.)),
    })
    .init(InitLifetimeModifier {
        // Give a bit of variation by randomizing the lifetime per particle
        lifetime: Value::Uniform((0.2, 0.4)),
    })
    .init(InitAgeModifier {
        // Give a bit of variation by randomizing the age per particle. This will control the
        // starting color and starting size of particles.
        age: Value::Uniform((0.0, 0.2)),
    })
    .update(LinearDragModifier { drag: 5. })
    // .update(AccelModifier::constant(Vec3::new(0., -8., 0.)))
    .render(ColorOverLifetimeModifier {
        gradient: color_gradient1,
    })
    .render(SizeOverLifetimeModifier {
        gradient: size_gradient1,
    })
}

fn hyper_sprint_effect() -> EffectAsset {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::splat(1.0));
    gradient.add_key(0.5, Vec4::splat(1.0));
    gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0));

    EffectAsset {
        name: "Gradient".to_string(),
        // TODO: Figure out why no particle spawns if this is 1
        capacity: 32768,
        spawner: Spawner::once(32.0.into(), false),
        ..Default::default()
    }
    .init(InitPositionCircleModifier {
        center: Vec3::Z * 0.1,
        axis: Vec3::Z,
        radius: PLAYER_R,
        dimension: ShapeDimension::Surface,
    })
    .init(InitVelocityCircleModifier {
        center: Vec3::ZERO,
        axis: Vec3::Z,
        speed: Value::Uniform((2.0, 3.0)),
    })
    .init(InitLifetimeModifier {
        lifetime: 0.5_f32.into(),
    })
    .render(ColorOverLifetimeModifier { gradient })
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::constant([0.2; 2].into()),
    })
}

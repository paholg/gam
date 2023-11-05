use bevy::{
    prelude::{
        default,
        shape::{self, Circle, Icosphere},
        AssetServer, Assets, Color, Commands, Component, Entity, Handle, Mesh, Res, ResMut,
        Resource, StandardMaterial, Vec2, Vec3, Vec4,
    },
    scene::Scene,
};
use bevy_hanabi::{
    Attribute, ColorOverLifetimeModifier, EffectAsset, ExprWriter, Gradient, LinearDragModifier,
    ParticleEffectBundle, SetAttributeModifier, SetPositionCircleModifier,
    SetPositionSphereModifier, SetVelocityCircleModifier, SetVelocitySphereModifier,
    ShapeDimension, SizeOverLifetimeModifier, Spawner,
};
use bevy_kira_audio::AudioSource;
use iyes_progress::prelude::AssetsLoading;

use engine::{
    ability::properties::{AbilityProps, GrenadeProps, GunProps},
    PLAYER_R,
};

use crate::{
    bar::{Energybar, Healthbar},
    shapes::HollowPolygon,
};

pub struct BarAssets {
    pub mesh: Handle<Mesh>,
    pub fg_material: Handle<StandardMaterial>,
    pub bg_material: Handle<StandardMaterial>,
}

pub struct ShotAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub effect_entity: Entity,
    pub spawn_sound: Handle<AudioSource>,
    pub despawn_sound: Handle<AudioSource>,
}

pub struct GrenadeAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub effect_entity: Entity,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
}

pub struct Target {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub struct HyperSprintAssets {
    pub effect_entity: Entity,
}

pub struct CharacterAssets {
    pub scene: Handle<Scene>,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
    pub despawn_sound: Handle<AudioSource>,
    pub despawn_effect: Entity,
}

// A collection of HandleIds for assets for spawning.
#[derive(Resource)]
pub struct AssetHandler {
    pub healthbar: BarAssets,
    pub energybar: BarAssets,
    pub shot: ShotAssets,
    pub frag_grenade: GrenadeAssets,
    pub heal_grenade: GrenadeAssets,
    pub hyper_sprint: HyperSprintAssets,
    pub player: CharacterAssets,
    pub ally: CharacterAssets,
    pub enemy: CharacterAssets,
    pub music: Vec<(String, Handle<AudioSource>)>,
    pub target: Target,
}

#[derive(Component)]
pub struct ShotEffect;

#[derive(Component)]
pub struct HyperSprintEffect;

#[derive(Component)]
pub struct DeathEffect;

#[derive(Component)]
pub struct FragGrenadeEffect;

pub fn asset_handler_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: ResMut<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    props: Res<AbilityProps>,
) {
    let fg = StandardMaterial {
        base_color: Color::GREEN,
        unlit: true,
        ..Default::default()
    };
    let bg = StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        ..Default::default()
    };
    let healthbar = BarAssets {
        mesh: meshes.add(
            shape::Quad {
                size: Healthbar::default().size,
                ..default()
            }
            .into(),
        ),
        fg_material: materials.add(fg),
        bg_material: materials.add(bg.clone()),
    };

    let fg = StandardMaterial {
        base_color: Color::RgbaLinear {
            red: 0.0,
            green: 0.2,
            blue: 0.8,
            alpha: 1.0,
        },
        unlit: true,
        ..Default::default()
    };
    let energybar = BarAssets {
        mesh: meshes.add(
            shape::Quad {
                size: Energybar::default().size,
                ..default()
            }
            .into(),
        ),
        fg_material: materials.add(fg),
        bg_material: materials.add(bg),
    };

    let effect = effects.add(shot_effect(&props.gun));
    let effect_entity = commands
        .spawn(ParticleEffectBundle::new(effect))
        .insert(ShotEffect)
        .id();

    let shot_material = StandardMaterial {
        emissive: Color::rgb_linear(0.0, 20.0, 20.0),
        ..Default::default()
    };

    let shot = ShotAssets {
        mesh: meshes.add(
            Mesh::try_from(Icosphere {
                radius: 1.0,
                subdivisions: 5,
            })
            .unwrap(),
        ),
        material: materials.add(shot_material),
        effect_entity,
        spawn_sound: asset_server.load("audio/laserSmall_000.ogg"),
        despawn_sound: asset_server.load("audio/laserSmall_000.ogg"),
    };
    loading.add(&shot.spawn_sound);
    loading.add(&shot.despawn_sound);

    let frag_grenade_material = StandardMaterial {
        emissive: Color::rgb_linear(10.0, 0.0, 0.1),
        ..Default::default()
    };

    let frag_grenade_effect = effects.add(frag_grenade_effect(&props.frag_grenade));
    let frag_effect_entity = commands
        .spawn(ParticleEffectBundle::new(frag_grenade_effect))
        .insert(FragGrenadeEffect)
        .id();

    let frag_grenade = GrenadeAssets {
        mesh: meshes.add(
            Mesh::try_from(Icosphere {
                radius: props.frag_grenade.radius,
                subdivisions: 5,
            })
            .unwrap(),
        ),
        material: materials.add(frag_grenade_material.clone()),
        outline_mesh: meshes.add(
            HollowPolygon {
                radius: props.frag_grenade.explosion_radius,
                thickness: 0.25,
                vertices: 60,
            }
            .into(),
        ),
        outline_material: materials.add(frag_grenade_material),
        effect_entity: frag_effect_entity,
    };

    let heal_grenade_material = StandardMaterial {
        emissive: Color::rgb_linear(0.0, 10.0, 0.1),
        ..Default::default()
    };

    let heal_grenade_effect = effects.add(heal_grenade_effect(&props.heal_grenade));
    let heal_effect_entity = commands
        .spawn(ParticleEffectBundle::new(heal_grenade_effect))
        .insert(FragGrenadeEffect)
        .id();

    let heal_grenade = GrenadeAssets {
        mesh: meshes.add(
            Mesh::try_from(Icosphere {
                radius: props.heal_grenade.radius,
                subdivisions: 5,
            })
            .unwrap(),
        ),
        outline_mesh: meshes.add(
            HollowPolygon {
                radius: props.heal_grenade.explosion_radius,
                thickness: 0.25,
                vertices: 60,
            }
            .into(),
        ),
        material: materials.add(heal_grenade_material.clone()),
        outline_material: materials.add(heal_grenade_material),
        effect_entity: heal_effect_entity,
    };

    let effect = effects.add(hyper_sprint_effect());
    let effect_entity = commands
        .spawn(ParticleEffectBundle::new(effect))
        .insert(HyperSprintEffect)
        .id();

    let hyper_sprint = HyperSprintAssets { effect_entity };

    let robot = asset_server.load("models/temp/robot1.glb#Scene0");
    loading.add(&robot);
    let snowman = asset_server.load("models/temp/snowman.glb#Scene0");
    loading.add(&snowman);
    let death_sound = asset_server.load("audio/explosionCrunch_000.ogg");
    loading.add(&death_sound);

    let outline = meshes.add(
        HollowPolygon {
            radius: 1.0,
            thickness: 0.15,
            vertices: 30,
        }
        .into(),
    );

    let outline_material = StandardMaterial {
        // FIXME: broken on windows with these settings
        // depth_bias: OUTLINE_DEPTH_BIAS,
        // perceptual_roughness: 1.0,
        // metallic: 1.0,
        // fog_enabled: false,
        // alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    };

    const OUTLINE_ALPHA: f32 = 0.5;

    let mut player_outline = outline_material.clone();
    player_outline.base_color = Color::GREEN.with_a(OUTLINE_ALPHA);
    player_outline.emissive = Color::GREEN.with_a(OUTLINE_ALPHA);
    let mut enemy_outline = outline_material.clone();
    enemy_outline.base_color = Color::RED.with_a(OUTLINE_ALPHA);
    enemy_outline.emissive = Color::RED.with_a(OUTLINE_ALPHA);
    let mut ally_outline = outline_material.clone();
    ally_outline.base_color = Color::CYAN.with_a(OUTLINE_ALPHA);
    ally_outline.emissive = Color::CYAN.with_a(OUTLINE_ALPHA);

    let death_effect = effects.add(death_effect());
    let death_effect_entity = commands
        .spawn(ParticleEffectBundle::new(death_effect))
        .insert(DeathEffect)
        .id();

    let player = CharacterAssets {
        scene: robot.clone(),
        outline_mesh: outline.clone(),
        outline_material: materials.add(player_outline),
        despawn_sound: death_sound.clone(),
        despawn_effect: death_effect_entity,
    };

    let ally = CharacterAssets {
        scene: robot,
        outline_mesh: outline.clone(),
        outline_material: materials.add(ally_outline),
        despawn_sound: death_sound.clone(),
        despawn_effect: death_effect_entity,
    };

    let enemy = CharacterAssets {
        scene: snowman,
        outline_mesh: outline,
        outline_material: materials.add(enemy_outline),
        despawn_sound: death_sound,
        despawn_effect: death_effect_entity,
    };

    let mut target_material = outline_material;
    target_material.base_color = Color::CRIMSON;
    target_material.emissive = Color::CRIMSON;
    let target = Target {
        mesh: meshes.add(Circle::new(0.2).into()),
        material: materials.add(target_material),
    };

    let asset_handler = AssetHandler {
        music: load_music(&asset_server, &mut loading),
        healthbar,
        energybar,
        shot,
        frag_grenade,
        heal_grenade,
        hyper_sprint,
        player,
        ally,
        enemy,
        target,
    };
    commands.insert_resource(asset_handler);
}

fn load_music(
    asset_server: &AssetServer,
    loading: &mut AssetsLoading,
) -> Vec<(String, Handle<AudioSource>)> {
    // Load all assets in parallel first
    asset_server
        .load_folder("audio/Galacti-Chrons Weird Music Pack")
        .unwrap();
    let mut res = Vec::new();
    for entry in glob::glob("assets/audio/Galacti-Chrons Weird Music Pack/*.ogg").unwrap() {
        if let Ok(path) = entry {
            let fname = path.file_name().unwrap().to_string_lossy().into_owned();
            let rel_path = format!("audio/Galacti-Chrons Weird Music Pack/{}", fname);
            let handle = asset_server.get_handle(rel_path);
            loading.add(&handle);
            res.push((fname, handle));
        }
    }
    res
}

fn shot_effect(props: &GunProps) -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(0.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.5, Vec4::new(2.0, 2.0, 4.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 4.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 4.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1));
    size_gradient1.add_key(0.3, Vec2::splat(0.1));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let spawner = Spawner::once(250.0.into(), false);
    let writer = ExprWriter::new();

    let pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(props.radius).expr(),
        dimension: ShapeDimension::Volume,
    };

    let vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(6.0).uniform(writer.lit(7.0)).expr(),
    };

    let lifetime = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(0.2).uniform(writer.lit(0.4)).expr(),
    };

    let age = SetAttributeModifier {
        attribute: Attribute::AGE,
        value: writer.lit(0.0).uniform(writer.lit(0.2)).expr(),
    };

    let drag = LinearDragModifier {
        drag: writer.lit(5.0).expr(),
    };

    EffectAsset::new(32768, spawner, writer.finish())
        .with_name("shot_particle_effect")
        .init(pos)
        .init(vel)
        .init(lifetime)
        .init(age)
        .update(drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
}

fn death_effect() -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 4.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.2));
    size_gradient1.add_key(0.3, Vec2::splat(0.3));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let spawner = Spawner::once(500.0.into(), false);
    let writer = ExprWriter::new();

    let pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(PLAYER_R).expr(),
        dimension: ShapeDimension::Volume,
    };

    let vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(6.0).uniform(writer.lit(7.0)).expr(),
    };

    let lifetime = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(0.4).uniform(writer.lit(0.6)).expr(),
    };

    let age = SetAttributeModifier {
        attribute: Attribute::AGE,
        value: writer.lit(0.0).uniform(writer.lit(0.2)).expr(),
    };

    let drag = LinearDragModifier {
        drag: writer.lit(5.0).expr(),
    };

    EffectAsset::new(32768, spawner, writer.finish())
        .with_name("death_effect")
        .init(pos)
        .init(vel)
        .init(lifetime)
        .init(age)
        .update(drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
}

fn frag_grenade_effect(props: &GrenadeProps) -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(0.6, Vec4::new(2.0, 1.0, 0.0, 1.0));
    color_gradient1.add_key(0.8, Vec4::new(0.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.2));
    size_gradient1.add_key(0.3, Vec2::splat(0.3));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let spawner = Spawner::once(500.0.into(), false);
    let writer = ExprWriter::new();

    let pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(props.explosion_radius).expr(),
        dimension: ShapeDimension::Volume,
    };

    let vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(6.0).uniform(writer.lit(7.0)).expr(),
    };

    let lifetime = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(0.4).uniform(writer.lit(0.6)).expr(),
    };

    let age = SetAttributeModifier {
        attribute: Attribute::AGE,
        value: writer.lit(0.0).uniform(writer.lit(0.2)).expr(),
    };

    let drag = LinearDragModifier {
        drag: writer.lit(5.0).expr(),
    };

    EffectAsset::new(32768, spawner, writer.finish())
        .with_name("frag_grenade_effect")
        .init(pos)
        .init(vel)
        .init(lifetime)
        .init(age)
        .update(drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
}

fn heal_grenade_effect(props: &GrenadeProps) -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(0.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.8, Vec4::new(0.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.2));
    size_gradient1.add_key(0.3, Vec2::splat(0.3));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let spawner = Spawner::once(500.0.into(), false);
    let writer = ExprWriter::new();

    let pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(props.explosion_radius).expr(),
        dimension: ShapeDimension::Volume,
    };

    let vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(6.0).uniform(writer.lit(7.0)).expr(),
    };

    let lifetime = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(0.4).uniform(writer.lit(0.6)).expr(),
    };

    let age = SetAttributeModifier {
        attribute: Attribute::AGE,
        value: writer.lit(0.0).uniform(writer.lit(0.2)).expr(),
    };

    let drag = LinearDragModifier {
        drag: writer.lit(5.0).expr(),
    };

    EffectAsset::new(32768, spawner, writer.finish())
        .with_name("heal_grenade_effect")
        .init(pos)
        .init(vel)
        .init(lifetime)
        .init(age)
        .update(drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
}

fn hyper_sprint_effect() -> EffectAsset {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::splat(1.0));
    gradient.add_key(0.5, Vec4::splat(1.0));
    gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0));

    let spawner = Spawner::once(32.0.into(), false);
    let writer = ExprWriter::new();

    let pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::Z * 0.1).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(PLAYER_R).expr(),
        dimension: ShapeDimension::Surface,
    };

    let vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        speed: writer.lit(2.0).uniform(writer.lit(3.0)).expr(),
    };

    let lifetime = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(0.5).expr(),
    };

    EffectAsset::new(32768, spawner, writer.finish())
        .with_name("hyper_sprint_effect")
        .init(pos)
        .init(vel)
        .init(lifetime)
        .render(ColorOverLifetimeModifier { gradient })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant([0.2; 2].into()),
            screen_space_size: false,
        })
}

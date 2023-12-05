use bevy::{
    asset::LoadedFolder,
    prelude::{
        default,
        shape::{self, Capsule, Circle, Cylinder, Icosphere},
        AssetServer, Assets, Color, Commands, Handle, Mesh, Res, ResMut, Resource,
        StandardMaterial, Vec2, Vec3, Vec4,
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
    ability::properties::{AbilityProps, GunProps},
    PLAYER_R,
};

use crate::{color_gradient::ColorGradient, particles::ParticleEffectPool, shapes::HollowPolygon};

pub struct BarAssets {
    pub mesh: Handle<Mesh>,
    pub fg_material: Handle<StandardMaterial>,
    pub bg_material: Handle<StandardMaterial>,
}

pub struct ShotAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub collision_effect: ParticleEffectPool,
    pub spawn_sound: Handle<AudioSource>,
    pub despawn_sound: Handle<AudioSource>,
}

pub struct GrenadeAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
    pub explosion: ExplosionAssets,
}

pub struct SeekerRocketAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub explosion: ExplosionAssets,
}

pub struct NeutrinoBallAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
}

pub struct TargetAssets {
    pub cursor_mesh: Handle<Mesh>,
    pub cursor_material: Handle<StandardMaterial>,
    pub laser_mesh: Handle<Mesh>,
    pub laser_material: Handle<StandardMaterial>,
    pub laser_length: f32,
}

pub struct HyperSprintAssets {
    pub effect: ParticleEffectPool,
}

pub struct CharacterAssets {
    pub scene: Handle<Scene>,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
    pub despawn_sound: Handle<AudioSource>,
    pub despawn_effect: ParticleEffectPool,
}

pub struct ExplosionAssets {
    pub gradient: ColorGradient,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    // TODO: Add a sound.
}

impl ExplosionAssets {
    fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        colors: ColorGradient,
    ) -> Self {
        let initial_color = colors.get(0.0);
        ExplosionAssets {
            gradient: colors,
            mesh: meshes.add(
                Mesh::try_from(Icosphere {
                    radius: 1.0,
                    subdivisions: 5,
                })
                .unwrap(),
            ),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.0, 0.0, 0.0, 0.5),
                emissive: initial_color,
                alpha_mode: bevy::prelude::AlphaMode::Blend,
                ..Default::default()
            }),
        }
    }
}

// A collection of HandleIds for assets for spawning.
#[derive(Resource)]
pub struct AssetHandler {
    pub healthbar: BarAssets,
    pub energybar: BarAssets,
    pub shot: ShotAssets,
    pub frag_grenade: GrenadeAssets,
    pub heal_grenade: GrenadeAssets,
    pub seeker_rocket: SeekerRocketAssets,
    pub neutrino_ball: NeutrinoBallAssets,
    pub hyper_sprint: HyperSprintAssets,
    pub player: CharacterAssets,
    pub ally: CharacterAssets,
    pub enemy: CharacterAssets,
    pub music: Handle<LoadedFolder>,
    pub target: TargetAssets,
}

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
                size: Vec2::new(1.0, 1.0),
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
        mesh: healthbar.mesh.clone(),
        fg_material: materials.add(fg),
        bg_material: materials.add(bg),
    };

    let effect = effects.add(shot_effect(&props.gun));
    let effect_pool = ParticleEffectBundle::new(effect).into();

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
        collision_effect: effect_pool,
        spawn_sound: asset_server.load("third-party/audio/other/laserSmall_000.ogg"),
        despawn_sound: asset_server.load("third-party/audio/other/laserSmall_000.ogg"),
    };
    loading.add(&shot.spawn_sound);
    loading.add(&shot.despawn_sound);

    let frag_grenade_material = StandardMaterial {
        emissive: Color::rgb_linear(10.0, 0.0, 0.1),
        ..Default::default()
    };

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
                radius: props.frag_grenade.explosion.max_radius,
                thickness: 0.06,
                vertices: 60,
            }
            .into(),
        ),
        outline_material: materials.add(frag_grenade_material),
        explosion: ExplosionAssets::new(
            &mut meshes,
            &mut materials,
            ColorGradient::new([
                (0.0, Color::rgba(5.0, 1.2, 0.0, 0.2)),
                (0.8, Color::rgba(10.0, 2.5, 0.0, 0.2)),
                (1.0, Color::rgba(0.0, 0.0, 0.0, 0.8)),
            ]),
        ),
    };

    let heal_grenade_material = StandardMaterial {
        emissive: Color::rgb_linear(0.0, 10.0, 0.1),
        ..Default::default()
    };

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
                radius: props.heal_grenade.explosion.max_radius,
                thickness: 0.06,
                vertices: 60,
            }
            .into(),
        ),
        material: materials.add(heal_grenade_material.clone()),
        outline_material: materials.add(heal_grenade_material),
        explosion: ExplosionAssets::new(
            &mut meshes,
            &mut materials,
            ColorGradient::new([
                (0.0, Color::rgba(0.0, 5.0, 0.0, 0.2)),
                (0.8, Color::rgba(0.0, 10.0, 0.0, 0.2)),
                (1.0, Color::rgba(0.0, 0.0, 0.0, 0.8)),
            ]),
        ),
    };

    let seeker_rocket_material = StandardMaterial {
        emissive: Color::rgb_linear(10.0, 0.1, 10.0),
        ..Default::default()
    };

    let seeker_rocket = SeekerRocketAssets {
        mesh: meshes.add(
            Mesh::try_from(Capsule {
                radius: props.seeker_rocket.capsule_radius,
                depth: props.seeker_rocket.capsule_length,
                ..Default::default()
            })
            .unwrap(),
        ),
        material: materials.add(seeker_rocket_material.clone()),
        explosion: ExplosionAssets::new(
            &mut meshes,
            &mut materials,
            ColorGradient::new([
                (0.0, Color::rgba(5.0, 0.1, 5.0, 0.2)),
                (0.8, Color::rgba(10.0, 0.1, 10.0, 0.2)),
                (1.0, Color::rgba(0.5, 0.05, 0.5, 0.8)),
            ]),
        ),
    };

    let neutrino_ball_material = StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        ..Default::default()
    };
    let neutrino_ball = NeutrinoBallAssets {
        mesh: meshes.add(
            Mesh::try_from(Icosphere {
                radius: props.neutrino_ball.radius,
                ..Default::default()
            })
            .unwrap(),
        ),
        material: materials.add(neutrino_ball_material.clone()),
        outline_mesh: meshes.add(
            HollowPolygon {
                radius: props.neutrino_ball.effect_radius,
                thickness: 0.06,
                vertices: 60,
            }
            .into(),
        ),
        outline_material: materials.add(neutrino_ball_material),
    };

    let effect = effects.add(hyper_sprint_effect());
    let hyper_sprint_effect = ParticleEffectBundle::new(effect).into();

    let hyper_sprint = HyperSprintAssets {
        effect: hyper_sprint_effect,
    };

    let robot = asset_server.load("models/temp/robot1.glb#Scene0");
    loading.add(&robot);
    let snowman = asset_server.load("models/temp/snowman.glb#Scene0");
    loading.add(&snowman);
    let death_sound = asset_server.load("third-party/audio/other/explosionCrunch_000.ogg");
    loading.add(&death_sound);

    let outline = meshes.add(
        HollowPolygon {
            radius: PLAYER_R,
            thickness: 0.04,
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
    let death_effect: ParticleEffectPool = ParticleEffectBundle::new(death_effect).into();

    let player = CharacterAssets {
        scene: robot.clone(),
        outline_mesh: outline.clone(),
        outline_material: materials.add(player_outline),
        despawn_sound: death_sound.clone(),
        despawn_effect: death_effect.clone(),
    };

    let ally = CharacterAssets {
        scene: robot,
        outline_mesh: outline.clone(),
        outline_material: materials.add(ally_outline),
        despawn_sound: death_sound.clone(),
        despawn_effect: death_effect.clone(),
    };

    let enemy = CharacterAssets {
        scene: snowman,
        outline_mesh: outline,
        outline_material: materials.add(enemy_outline),
        despawn_sound: death_sound,
        despawn_effect: death_effect,
    };

    let target_material = StandardMaterial {
        emissive: Color::rgb_linear(10.0, 0.0, 0.1),
        ..Default::default()
    };

    let target_laser_material = StandardMaterial {
        emissive: Color::rgb_linear(10.0, 0.0, 0.1),
        ..Default::default()
    };
    let laser_length = 100.0;
    let target = TargetAssets {
        cursor_mesh: meshes.add(Circle::new(0.06).into()),
        cursor_material: materials.add(target_material),
        laser_mesh: meshes.add(
            Cylinder {
                radius: 0.01,
                height: 1.0,
                resolution: 3,
                segments: 1,
            }
            .into(),
        ),
        laser_material: materials.add(target_laser_material),
        laser_length,
    };

    let asset_handler = AssetHandler {
        music: load_music(&asset_server),
        healthbar,
        energybar,
        shot,
        frag_grenade,
        heal_grenade,
        seeker_rocket,
        neutrino_ball,
        hyper_sprint,
        player,
        ally,
        enemy,
        target,
    };
    commands.insert_resource(asset_handler);
}

fn load_music(asset_server: &AssetServer) -> Handle<LoadedFolder> {
    asset_server.load_folder("third-party/audio/Galacti-Chrons Weird Music Pack")
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

    let spawner = Spawner::once(250.0.into(), true);
    let writer = ExprWriter::new();

    let pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(props.radius).expr(),
        dimension: ShapeDimension::Volume,
    };

    let vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(1.5).uniform(writer.lit(2.0)).expr(),
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
    size_gradient1.add_key(0.0, Vec2::splat(0.05));
    size_gradient1.add_key(0.3, Vec2::splat(0.07));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let spawner = Spawner::once(500.0.into(), true);
    let writer = ExprWriter::new();

    let pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(PLAYER_R).expr(),
        dimension: ShapeDimension::Volume,
    };

    let vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(1.5).uniform(writer.lit(2.0)).expr(),
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

fn hyper_sprint_effect() -> EffectAsset {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::splat(1.0));
    color_gradient.add_key(0.5, Vec4::splat(1.0));
    color_gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0));

    let spawner = Spawner::once(32.0.into(), true);
    let writer = ExprWriter::new();

    let pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::Y * 0.1).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(PLAYER_R).expr(),
        dimension: ShapeDimension::Surface,
    };

    let vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        speed: writer.lit(0.5).uniform(writer.lit(0.8)).expr(),
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
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant([0.05; 2].into()),
            screen_space_size: false,
        })
}

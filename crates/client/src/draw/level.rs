use bevy::{
    asset::Assets,
    color::{
        palettes::css::{ALICE_BLUE, AQUAMARINE, RED},
        Alpha, Color,
    },
    ecs::{resource::Resource, system::ResMut},
    math::primitives::Cuboid,
    pbr::MeshMaterial3d,
    prelude::{
        Added, Commands, Component, Entity, Handle, InheritedVisibility, Mesh3d, Query, Res,
        SpotLight, StandardMaterial, Transform, Vec3,
    },
    render::{alpha::AlphaMode, mesh::Mesh},
};
use engine::{
    level::{Floor, InLevel, LevelProps, SHORT_WALL, WALL_HEIGHT},
    lifecycle::DEATH_Y,
    UP,
};

use crate::aim::BlocksSight;

#[derive(Resource)]
pub struct WallAssets {
    pub shape: Handle<Mesh>,
    pub floor: Handle<StandardMaterial>,
    pub short_wall: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub tall_wall: Handle<StandardMaterial>,
    pub short_wall_trans: Handle<StandardMaterial>,
    pub wall_trans: Handle<StandardMaterial>,
    pub tall_wall_trans: Handle<StandardMaterial>,
}

impl WallAssets {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let short_wall_color = ALICE_BLUE.into();
        let wall_color = AQUAMARINE.into();
        let tall_wall_color = RED.into();

        let trans = |color: Color| StandardMaterial {
            base_color: color.with_alpha(0.5),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        };

        WallAssets {
            shape: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            floor: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.6, 0.1),
                perceptual_roughness: 0.8,
                ..Default::default()
            }),
            short_wall: materials.add(short_wall_color),
            wall: materials.add(wall_color),
            tall_wall: materials.add(tall_wall_color),
            short_wall_trans: materials.add(trans(short_wall_color)),
            wall_trans: materials.add(trans(wall_color)),
            tall_wall_trans: materials.add(trans(tall_wall_color)),
        }
    }
}

pub fn create_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(WallAssets::new(&mut meshes, &mut materials));
}

#[derive(Component, Copy, Clone)]
pub enum WallKind {
    Floor,
    Short,
    Standard,
    Tall,
}

impl WallKind {
    fn opaque(&self, assets: &WallAssets) -> Handle<StandardMaterial> {
        match self {
            WallKind::Floor => assets.floor.clone_weak(),
            WallKind::Short => assets.short_wall.clone_weak(),
            WallKind::Standard => assets.wall.clone_weak(),
            WallKind::Tall => assets.tall_wall.clone_weak(),
        }
    }

    fn trans(&self, assets: &WallAssets) -> Handle<StandardMaterial> {
        match self {
            WallKind::Floor => assets.floor.clone_weak(), // no trans floor
            WallKind::Short => assets.short_wall_trans.clone_weak(),
            WallKind::Standard => assets.wall_trans.clone_weak(),
            WallKind::Tall => assets.tall_wall_trans.clone_weak(),
        }
    }

    fn is_wall(&self) -> bool {
        match self {
            WallKind::Floor => false,
            WallKind::Short | WallKind::Standard | WallKind::Tall => true,
        }
    }
}

#[derive(Component)]
pub struct Wall;

pub fn draw_wall_system(
    mut commands: Commands,
    assets: Res<WallAssets>,
    query: Query<(Entity, &Floor), Added<Floor>>,
) {
    for (entity, floor) in &query {
        let kind = if floor.dim.y >= WALL_HEIGHT - DEATH_Y + 0.1 {
            WallKind::Tall
        } else if floor.dim.y >= WALL_HEIGHT - DEATH_Y - 0.1 {
            WallKind::Standard
        } else if floor.dim.y >= SHORT_WALL - DEATH_Y - 0.1 {
            WallKind::Short
        } else {
            WallKind::Floor
        };

        // Add InheritedVisibility to make bevy happy.
        commands
            .entity(entity)
            .insert(InheritedVisibility::default());

        // We want to chunk walls into a "floor" section and a "wall" section, so
        // we're only making the part above the floor transparent when it's
        // blocking a character.
        let props = if kind.is_wall() {
            let mut floor_scale = floor.dim;
            floor_scale.y = -DEATH_Y;

            let mut wall_scale = floor.dim;
            wall_scale.y = floor.dim.y + DEATH_Y;
            vec![
                (
                    Transform::from_scale(floor_scale).with_translation(Vec3::new(
                        0.0,
                        -wall_scale.y * 0.5,
                        0.0,
                    )),
                    WallKind::Floor,
                ),
                (
                    Transform::from_scale(wall_scale).with_translation(Vec3::new(
                        0.0,
                        floor_scale.y * 0.5,
                        0.0,
                    )),
                    kind,
                ),
            ]
        } else {
            vec![(Transform::from_scale(floor.dim), kind)]
        };

        let ids = props
            .into_iter()
            .map(|(transform, kind)| {
                let wall = commands
                    .spawn((
                        Mesh3d(assets.shape.clone_weak()),
                        MeshMaterial3d(kind.opaque(&assets)),
                        transform,
                        kind,
                        BlocksSight,
                    ))
                    .id();
                if kind.is_wall() {
                    commands.entity(wall).insert(Wall);
                }
                wall
            })
            .collect::<Vec<_>>();

        commands.entity(entity).add_children(&ids);
    }
}

pub fn draw_lights_system(
    mut commands: Commands,
    level: Res<LevelProps>,
    query: Query<&SpotLight>,
) {
    if query.iter().next().is_some() {
        return;
    }
    let altitude = 10.0;
    let spacing = 15.0;

    let nx = (level.x / spacing).ceil().max(1.0) as usize;
    let nz = (level.z / spacing).ceil().max(1.0) as usize;

    let offset = |n| {
        if n % 2 == 0 {
            let offset = ((n as f32) * 0.5 - 1.0) * spacing + spacing * 0.5;
            -offset
        } else {
            let offset = (n as f32 - 1.0) * 0.5 * spacing;
            -offset
        }
    };

    let xoffset = offset(nx);
    let zoffset = offset(nz);

    for x in 0..nx {
        for z in 0..nz {
            let x = (x as f32) * spacing + xoffset;
            let z = (z as f32) * spacing + zoffset;

            // Offset the light a bit, for more interesting shadows.
            let transform = Transform::from_xyz(x - spacing * 0.5, altitude, z - spacing)
                .looking_at(Vec3::new(x, 0.0, z), UP);

            commands.spawn((
                SpotLight {
                    // FIXME: We definitely want shadows, but they're so buggy that they're more
                    // distracting than helpful at the moment.
                    shadows_enabled: false,
                    range: 30.0,
                    intensity: 4_000_000.0,
                    outer_angle: std::f32::consts::FRAC_PI_3,
                    inner_angle: 0.0,
                    ..Default::default()
                },
                transform,
                InLevel,
            ));
        }
    }
}

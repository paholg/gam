use bevy::pbr::MeshMaterial3d;
use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::GlobalTransform;
use bevy::prelude::Handle;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Mesh3d;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::SpotLight;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Vec2;
use bevy::prelude::Vec3;
use bevy::prelude::With;
// FIXME
// use bevy_mod_raycast::prelude::RaycastMesh;
use engine::level::Floor;
use engine::level::InLevel;
use engine::level::LevelProps;
use engine::level::SHORT_WALL;
use engine::level::WALL_HEIGHT;
use engine::lifecycle::DEATH_Y;
use engine::Health;
use engine::To2d;
use engine::UP;

use crate::asset_handler::AssetHandler;
use crate::bar::Bar;

#[derive(Component, Copy, Clone)]
pub enum WallKind {
    Floor,
    Short,
    Standard,
    Tall,
}

impl WallKind {
    fn opaque(&self, assets: &AssetHandler) -> Handle<StandardMaterial> {
        match self {
            WallKind::Floor => assets.wall.floor.clone(),
            WallKind::Short => assets.wall.short_wall.clone(),
            WallKind::Standard => assets.wall.wall.clone(),
            WallKind::Tall => assets.wall.tall_wall.clone(),
        }
    }

    fn trans(&self, assets: &AssetHandler) -> Handle<StandardMaterial> {
        match self {
            WallKind::Floor => assets.wall.floor.clone(), // no trans floor
            WallKind::Short => assets.wall.short_wall_trans.clone(),
            WallKind::Standard => assets.wall.wall_trans.clone(),
            WallKind::Tall => assets.wall.tall_wall_trans.clone(),
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
    assets: Res<AssetHandler>,
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
                        Mesh3d(assets.wall.shape.clone_weak()),
                        MeshMaterial3d(kind.opaque(&assets)),
                        transform,
                        kind,
                    ))
                    .id();
                // TODO: Raycast
                // if kind.is_wall() {
                //     commands
                //         .entity(wall)
                //         .insert((Wall, RaycastMesh::<()>::default()));
                // }
                wall
            })
            .collect::<Vec<_>>();

        commands.entity(entity).add_children(&ids);
    }
}

pub fn update_wall_system(
    assets: Res<AssetHandler>,
    mut query: Query<
        (
            &mut MeshMaterial3d<StandardMaterial>,
            &Transform,
            &GlobalTransform,
            &WallKind,
        ),
        With<Wall>,
    >,
    healthbar_q: Query<(&GlobalTransform, &Bar<Health>)>,
) {
    const DELTA_Y: f32 = 1.3;

    struct BarInfo {
        loc: Vec2,
        size: Vec2,
    }

    let healthbars = healthbar_q
        .iter()
        .map(|(gt, bar)| BarInfo {
            loc: gt.translation().to_2d(),
            size: bar.size,
        })
        .collect::<Vec<_>>();
    // TODO: This is really inefficient.
    for (mut material, transform, global_transform, kind) in &mut query {
        let loc = global_transform.translation().to_2d();
        let shape = transform.scale.to_2d();

        let wall_left = loc.x - shape.x * 0.5;
        let wall_right = loc.x + shape.x * 0.5;
        let wall_top = loc.y + shape.y * 0.5;

        if healthbars.iter().any(|hb| {
            let hb_left = hb.loc.x - hb.size.x * 0.5;
            let hb_right = hb.loc.x + hb.size.x * 0.5;
            let hb_bottom = hb.loc.y + hb.size.y * 0.5;

            // Check if this bar is being blocked visually by this wall.
            hb_left < wall_right
                && hb_right > wall_left
                && hb_bottom > wall_top
                && hb_bottom < wall_top + DELTA_Y
        }) {
            *material = kind.trans(&assets).into();
        } else {
            *material = kind.opaque(&assets).into();
        }
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
                    shadows_enabled: true,
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

use bevy::asset::LoadedFolder;
use bevy::prelude::AssetServer;
use bevy::prelude::Assets;
use bevy::prelude::Commands;
use bevy::prelude::Handle;
use bevy::prelude::Mesh;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::prelude::StandardMaterial;
use bevy_hanabi::EffectAsset;
use iyes_progress::prelude::AssetsLoading;

use self::bar::BarAssets;
use self::character::CharacterAssets;
use self::music::load_music;
use self::target::TargetAssets;
use self::temperature::TemperatureAssets;
use self::time_dilation::TimeDilationAssets;
use self::wall::WallAssets;

pub mod bar;
pub mod character;
pub mod music;
pub mod target;
pub mod temperature;
pub mod time_dilation;
pub mod wall;

pub struct Builder<'a> {
    meshes: ResMut<'a, Assets<Mesh>>,
    materials: ResMut<'a, Assets<StandardMaterial>>,
    effects: ResMut<'a, Assets<EffectAsset>>,
    asset_server: ResMut<'a, AssetServer>,
    loading: ResMut<'a, AssetsLoading>,
}

// A collection of HandleIds for assets for spawning.
#[derive(Resource)]
pub struct AssetHandler {
    pub healthbar: BarAssets,
    pub energybar: BarAssets,
    pub time_dilation: TimeDilationAssets,
    pub player: CharacterAssets,
    pub ally: CharacterAssets,
    pub enemy: CharacterAssets,
    pub music: Handle<LoadedFolder>,
    pub target: TargetAssets,
    pub wall: WallAssets,
    pub temperature: TemperatureAssets,
}

impl<'a> Builder<'a> {
    fn build(&mut self) -> AssetHandler {
        AssetHandler {
            music: load_music(self),
            healthbar: BarAssets::healthbar(self),
            energybar: BarAssets::energybar(self),
            time_dilation: TimeDilationAssets::new(self),
            player: CharacterAssets::player(self),
            ally: CharacterAssets::ally(self),
            enemy: CharacterAssets::enemy(self),
            target: TargetAssets::new(self),
            wall: WallAssets::new(self),
            temperature: TemperatureAssets::new(self),
        }
    }
}

pub fn asset_handler_setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    effects: ResMut<Assets<EffectAsset>>,
    asset_server: ResMut<AssetServer>,
    loading: ResMut<AssetsLoading>,
) {
    let asset_handler = Builder {
        meshes,
        materials,
        effects,
        asset_server,
        loading,
    }
    .build();

    commands.insert_resource(asset_handler);
}

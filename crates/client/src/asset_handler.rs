use bevy::{
    asset::LoadedFolder,
    prelude::{
        AssetServer, Assets, Commands, Handle, Mesh, ResMut, Resource, StandardMaterial,
    },
};
use bevy_hanabi::EffectAsset;
use iyes_progress::prelude::AssetsLoading;

use self::{
    bar::BarAssets, character::CharacterAssets, music::load_music, target::TargetAssets,
    temperature::TemperatureAssets, time_dilation::TimeDilationAssets, transport::TransportAssets,
    wall::WallAssets,
};

pub mod bar;
pub mod character;
pub mod explosion;
pub mod music;
// pub mod rocket;
pub mod target;
pub mod temperature;
pub mod time_dilation;
pub mod transport;
pub mod wall;

pub struct Builder<'a> {
    meshes: ResMut<'a, Assets<Mesh>>,
    materials: ResMut<'a, Assets<StandardMaterial>>,
    effects: ResMut<'a, Assets<EffectAsset>>,
    asset_server: ResMut<'a, AssetServer>,
    loading: ResMut<'a, AssetsLoading>,
    // props: Res<'a, AbilityProps>,
}

// A collection of HandleIds for assets for spawning.
#[derive(Resource)]
pub struct AssetHandler {
    pub healthbar: BarAssets,
    pub energybar: BarAssets,
    // pub seeker_rocket: SeekerRocketAssets,
    pub time_dilation: TimeDilationAssets,
    pub player: CharacterAssets,
    pub ally: CharacterAssets,
    pub enemy: CharacterAssets,
    pub music: Handle<LoadedFolder>,
    pub target: TargetAssets,
    pub wall: WallAssets,
    pub transport: TransportAssets,
    pub temperature: TemperatureAssets,
}

impl<'a> Builder<'a> {
    fn build(&mut self) -> AssetHandler {
        AssetHandler {
            music: load_music(self),
            healthbar: BarAssets::healthbar(self),
            energybar: BarAssets::energybar(self),
            // seeker_rocket: SeekerRocketAssets::new(self),
            time_dilation: TimeDilationAssets::new(self),
            player: CharacterAssets::player(self),
            ally: CharacterAssets::ally(self),
            enemy: CharacterAssets::enemy(self),
            target: TargetAssets::new(self),
            wall: WallAssets::new(self),
            transport: TransportAssets::new(self),
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
    // props: Res<AbilityProps>,
) {
    let asset_handler = Builder {
        meshes,
        materials,
        effects,
        asset_server,
        loading,
        // props,
    }
    .build();

    commands.insert_resource(asset_handler);
}

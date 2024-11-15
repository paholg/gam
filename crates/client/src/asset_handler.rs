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
// use self::grenade::GrenadeAssets;
use self::music::load_music;
// use self::neutrino_ball::NeutrinoBallAssets;
// use self::rocket::SeekerRocketAssets;
use self::target::TargetAssets;
use self::temperature::TemperatureAssets;
use self::time_dilation::TimeDilationAssets;
// use self::transport::TransportAssets;
use self::wall::WallAssets;

pub mod bar;
pub mod character;
// pub mod explosion;
// pub mod grenade;
pub mod music;
// pub mod neutrino_ball;
// pub mod rocket;
pub mod target;
pub mod temperature;
pub mod time_dilation;
// pub mod transport;
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
    // pub frag_grenade: GrenadeAssets,
    // pub heal_grenade: GrenadeAssets,
    // pub seeker_rocket: SeekerRocketAssets,
    // pub neutrino_ball: NeutrinoBallAssets,
    pub time_dilation: TimeDilationAssets,
    pub player: CharacterAssets,
    pub ally: CharacterAssets,
    pub enemy: CharacterAssets,
    pub music: Handle<LoadedFolder>,
    pub target: TargetAssets,
    pub wall: WallAssets,
    // pub transport: TransportAssets,
    pub temperature: TemperatureAssets,
}

impl<'a> Builder<'a> {
    fn build(&mut self) -> AssetHandler {
        AssetHandler {
            music: load_music(self),
            healthbar: BarAssets::healthbar(self),
            energybar: BarAssets::energybar(self),
            // frag_grenade: GrenadeAssets::frag(self),
            // heal_grenade: GrenadeAssets::heal(self),
            // seeker_rocket: SeekerRocketAssets::new(self),
            // neutrino_ball: NeutrinoBallAssets::new(self),
            time_dilation: TimeDilationAssets::new(self),
            player: CharacterAssets::player(self),
            ally: CharacterAssets::ally(self),
            enemy: CharacterAssets::enemy(self),
            target: TargetAssets::new(self),
            wall: WallAssets::new(self),
            // transport: TransportAssets::new(self),
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

use bevy::app::App;
use bevy::MinimalPlugins;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(engine::GamPlugin)
        .run();
}

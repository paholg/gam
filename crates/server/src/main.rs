use bevy_app::App;
use bevy_internal::prelude::MinimalPlugins;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(engine::GamPlugin)
        .run();
}

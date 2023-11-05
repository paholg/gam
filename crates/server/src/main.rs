use bevy_app::App;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(engine::GamPlugin)
        .run();
}

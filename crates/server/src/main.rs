use bevy_app::App;

fn main() {
    let mut app = App::new();

    app.add_plugins(engine::GamPlugin);
    app.run();
}

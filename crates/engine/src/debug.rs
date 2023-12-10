use bevy_ecs::system::Resource;

/// Text used for debugging; set this in a system, and the `DebugTextPlugin`
/// from client to see it.
///
/// We intentionally do not add this resource inside the engine, so that debugs
/// won't be accidentally left in.
#[derive(Resource, Default)]
pub struct DebugText {
    val: String,
}

impl DebugText {
    pub fn set(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.val != text {
            tracing::info!("{}", self.val);
        }
        self.val = text;
    }

    pub fn get(&self) -> String {
        self.val.clone()
    }
}

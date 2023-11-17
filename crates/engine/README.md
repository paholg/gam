# Gam engine

The core game engine for this game. Suitable for inclusion in the game client,
server, or ai training, as it makes no use of graphics or user input.

## Determinism

The goal for this crate is to be fully cross-platform determinisic. Some gotchas
here:

1. Math: Rather than using functions like `sin` and `sqrt` from the std library,
   we need to use libm. See this note: https://rapier.rs/docs/user_guides/rust/determinism/

2. Events: See https://github.com/bevyengine/bevy/issues/7691. Note, while there
   is a workaround for custom events, there is not for built-in ones. So, we are
   kind of SoL for removing things, until this is done or we implement our own
   runner (see my comment in that issue).

3. GlobalTransform: https://github.com/bevyengine/bevy/issues/7836

4. Query order: https://github.com/bevyengine/bevy/issues/1470
   We'll likely have to implement something ourselves for now.

5. Systems: Right now, all of our systems are set to run in a strict order. We
   have to keep that for new systems, unless we are sure that they can't affect
   eachother.

6. Testing - definitely need a test that runs all systems for a while and hashes
   the world state, run on CI on all platforms we care about.

7. Inputs, not relevant to this crate, but relevant to this game: https://github.com/bevyengine/bevy/issues/6183

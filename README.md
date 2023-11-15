# Gam

A _very_ work-in-progress game. It doesn't even have a name yet.

## Ramblings
This is a cooperative game about a team of robots fighting aliens. I have some
more story ideas, but would not like to post them at this time.

It should be playable solo, or with up to 4-5 (not sure of max yet) players.

These are just some thoughts I have about the game, subject to change at any
time. Most of this is thought and not in the game at this time.

### Multiplayer
I'm not super excited about any server crates out there, and so will likely
write my own, inspired by ggrs, but designed as an authoritative server. Desired
properties:

* Ideally, the only data transmitted are initial conditions and player input.
* Game will run at a constant tick (probably 60 fps, because why not). To this
  end, all game logic is in its own crate `engine`.
* Client predicition with rollback. If we can achieve complete cross-platform
  determinism, then the server can just act as a spectator that relays messages
  and records results.
* Server should not rollback (this would be one downside to using ggrs); I see
  no reason why it can't just have a delayed view of the game.
* Reconnects, late joins, etc. should all be possible (another downside to
  ggrs).

### AI
I've experimented a bit with deep reinforcement learning; having an interesting
and challenging ai without having to program it sound really nice. I have not
had much luck so far.

Likely, I'll have to do a more traditional AI, at least initially.

If we get reinforcement learning working, likely we won't be able to in a cross-
platform deterministic way, so this will be a thing just the server does.

### Metagame
My idea is a classless RPG-style, with some metaprogression, and high build
diversity.
* By classless, I don't mean roleless. I would still like players to be able to
  specialize.
* Abilities will likely be tied to body parts. You play as a robot. Want a
  certain gun? You can make it your right arm. I think 5-6 abilities is a good
  area, plus maybe some passives.
  - Body parts that act as abilites: 2 arms, 1 legs, 1 body, 1 head maybe?
    - This gives a good excuse for giving everyone one "mobility" ability.
    - The two arms would be your primary attacks/abilities. Guns, melee weapons,
      heals should all fit here.
    - It might make sense for body to control passive(s) instead of an active
      ability. For example, choose a size to be big and high health or small and
      low health.
    - Another thought for passives: Everyone gets a small fixed energy regen.
      How you get energy on top of that is up to you:
      * Just increase the passive regen -- this may pair well with a support
        role.
      * "Steal" energy -- this could pair well with dps. Maybe it makes more
        sense as part of an ability though?
      * Regen from taking damage -- could be good if proper tanking roles
        develop.
      * Others?
* Progression. You should be able to pretty quickly try out different things,
  fully unlock a single "spec" with a moderate amount of time, and probably take
  quite a while to unlock _everything_.
  - You unlock an ability, and then each ability comes with its own skill tree.
  - Thinking that nothing in the tree is a "pure" upgrade, but that it functions
    to specialize the ability. Though this may be too hard.

### Cosmetics
As each ability is a body part, your look should reflect your spec. In addition,
it would be nice to offer varieties for each ability. However, given my present
3d-modeling non-ability, this may pose challenging.

### Gameplay
Isometric, action RPG style play, where you go on runs that should last about
30 minutes (I find this a sweet spot for "engaging enough" without being too
much of a time sink).
* No targeted abilities -- everything is a skill shot.
* Abilities don't have a team. What I mean by this is a bullet doesn't care who
  shot it; if it hits something, it does damage. Same goes for heals, buffs,
  etc. Whomever is hit with an ability takes its full effect.
  - This means friendly fire for enemies as well -- a good player should be able
    to abuse this, but AIs shouldn't just hit eachother randomly through normal
    play.
* Energy as a primary resource for everyone
  - One thought for build diversity is that everyone gets a small regen rate,
    with the ability to regen from additional sources.
    * Bump the regen rate -- this could be good for support roles.
    * Regen through damage -- could be good for dps, though it might make more
      sense as part of specific abilities rather than a global passive.
    * Regen though taking damage -- good if a proper tanking role develops.
* Health regen
  - Currently, we have the Heal Grenade that can get you back to full health,
    though its HP/energy ratio is much lower than damage-dealing abilities.
  - Should every player have some health regen ability? Maybe, like energy, we
    offer a passive baseline regen.
  - Alternatively, we could offer a "Repair" ability to everyone. You stop, and
    heal up. The idea being you would (usually, or maybe always) us this only
    between combats, allowing a spec to work with no heal abilities.
  - Auto-regen out-of-combat. I don't like differentiating "combat" from not;
    this is probably out.
  - We could limit healing in combat, by having a "temporary max health" that
    goes down, and resets between fights. I don't love that, but maybe you could
    use the aforementioned repair ability instead to restore your max health?

### Levels
For this, I could see going a few different ways.
* Full Procedural generation
* Pre-designed levels
* A mixture
  - This is likely the direction we will go, but how proportional? I like the
    idea of having boss-fights and/or objectives, which may be easier with
    pre-created at least sections.
* In-run progression.
  - Your build should definitely be determined between runs, but do we also want
    to have some in-run routelitey progression?
  - Maybe we have limited ammo rocket launchers, or other sorts of upgrades you
    can find in a level.
  - Maybe make things that fit with certain roles better than others? So for a
    coop run, you can make decisions about who gets what.
  - If we do offer this, things should not be pure upgrades.
  - We also likely don't want to get into the land of crazy builds that
    rougeli[tk]es do. Or maybe we do?

### Competitiveness
* This game will feature at least a leaderboard, but ideally more competitive
  features.
* If we go with procedural levels, then we should have a sort of "general"
  leaderboard, but also like a specific weekly seed.
* My current thoughts are that the leaderboard is ranked by difficulty, deaths,
  and time. Top of the leaderboard is fastest time through the hardest
  difficulty with zero deaths.
* Probably also a ladder system.

### Abilities
We should provide a good variety of abilities. Not all of them need to be
highly competitive, but they should all have uses and be fun to use. Currently,
we have (these are by no means final):
* Gun - simple projectile shooter.
  - Reloading guns doesn't exist yet, but I like the idea of magazines. For
    example, your gun holds 10 bullets and it takes 30 energy to reload. Reload
    when it's half full? Still takes 30 energy.
* Shotgun - shoots more projectiles at once.
* Grenade - throw to your cursor, explodes after a set time.
* Heal Grenade - same, but heals.
* Hypersprint - makes you go very fast, but drains a ton of energy while you
  hold it done.
Thoughts on others:
* Melee weapons - definitely want melee builds to be viable.
  - Generic slashy-thing
  - We could provide some melee-only affects, like energy-steal or health-steal
  - Electrified weapon? Maybe offer a melee weapon that you can
* Gravity orb - pulls things in. Do we throw this like a grenade? Shoot it like
  a slow-moving gun? I'm thinking it won't have collision detection -- think of
  it as a dense ball of dark matter; all it does is gravity. Maybe it flies
  straight and stops at your cursor?
* Seeking rocket - a rocket launcher, where the rocket tracks your cursor.
* Pushes/pulls - there are tons of abilities in this space; knockbacks, hooks,
  etc.
* Other movement things:
  - A jump could make you temporarily invulnerable, as nothing shoots up.
  - A portal could be fun, but might not be worth the implementation difficulty.
  - Teleport is always good.
* Energy transfer. Being able to give your energy to allies could play into some
  intersting things.
* Heals - we have the heal grenade, but should have other options, both for self
  heal, and for healing others.
  - I like the idea of a heal-beam.
* Pets. One build thought is a support/pet build, where you can e.g. build
  turrets, and focus on healing/giving them energy.
  - Turrets are an easy go to.
    * We could offer a lot of customization here, where maybe you can just pick
      an ability for a turret.
* Terrain manipulation.
  - Build walls -- temporary? permanent with health?
  - Destroy walls? Hard to think this wouldn't be permanent, maybe not good.
    Also would require a whole sort of framework for environment destruction.
    Probably nothing like this for a long time.
* Shields
  - Planting a shield in the ground might overlap with building a wall. Maybe
    this could be ranged?
  - Something that can attach to a character and give them temporary health.
* Cones - flamethrower, cold gun, etc.
* Slow field - time inside moves slower.
  - Alternatively, fast field?
* Lasers - lasers are good.
* "From Above" abilities. Every ability has to originate somewhere. Most
  originate from the character, though I could see some that perform action at
  a distance, like terrain manipulation. To do an AoE not at yourself, we can
  have things like grenades or rockets. But what about something that just
  happens at a spot? Do we want this? My only thought is e.g. ship-launched
  weapons.
  - Do we disable such an ability underground/inside?
  - Maybe we make abilities like this, but make them part of a level?

#### Ability interactions
Currently, when bullets collide, they both explode. Grenades just tank it. I
don't think one bullet should be enought to take out a grenade or a rocket, but
should they have health? Some other mechanic to explode early?

### Status Effects
Most of the abilities I've mentioned have "direct" effects, but I also like
persistent status-effects. This also seems like a good space for building
combos; use a "water gun" and a lightning something for damage, and also to help
burning allies. Too many seems like it could be hard to manage, though.
* Temperature - flame effects get you hot, cold effects bring you down.
  - Too hot: Catch fire and take damage over time.
  - Too cold: Start to freeze and slow down. Slow just movement? Maybe also
    lower enegy regen?
  - This allows fire/cold abilities to be used both offensively and defensively.
    Are there other status effects that we could have a similar spectrum for?
    If we just had 2 or 3 like this, that would probably be all the status
    effects we need.
* Oil - makes ground slippery, increases the ability to catch fire.
* Acid - makes you take extra damage.
* Water - take extra lightning damage.
* We're in sci-fi, so we can make some new shit up. Maybe a density changer?
  - Increase density: You're slower, but take less damage.
  - Decrease density: You're faster, but take more damage.
  - Also affects getting pushed/pulled around
* Similarly, mabye something that affects hardness? How hard/soft you are
  affects how much damage you take from different types? Do we want to have
  different damage types/resistances? That leads to a whole thing, and I'm not
  sure would be worth it from a player perspective.

### Controls
Game is currently controllable with KB+M and Gamepad -- these should both be
regarded as first-class citizens. Currently, we have a virtual cursor for the
gamepad, with a hard-coded max range. The range only presently affects grenades,
but do we want to have a configurable max range? A max range that applies to
mouse as well?

For grenades, there's a practical max, where it explodes before it lands. But
for other abilities there may not be.

It should not need to be said, but full control customization is a must.

It might make sense to provide two bindings for arm abilities. For example, on
a gun you should have shoot and reload. Maybe not all abilities will make use
of the second binding; if "reload" is our baseline, it's hard to think of a good
secondary action for abilities in general that would share a similar keybinding.
For example, for a gun you may want
* Click to shoot, R to reload on KBM
* Trigger to shoot, one of ABXY to reload on gamepad
What secondary do we provide a melee weapon, for example, that you would
want similarly out-of-the-way? Maybe a "change mode" button? Maybe throw or
something? How about a heal? I haven't thought too much about what should be arm
abilities vs others, so not sure what else goes here.

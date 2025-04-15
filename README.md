# YAGE
Yet Another Game Engine

This game engine is attempting to be a more data-oriented design.

for example,

one could write (simplified):
```rust

use yage::prelude::*;

manager! {
  struct Player {
    pos -> Vec3,
    delta -> Vec3,
  }
}

manager! {
  struct Enemy {
    weapon -> WeaponKind,
    mesh -> Mesh2D,
    pos -> Vec3,
    delta -> Vec3
  }
}

struct Game {
  core: EngineCore,
  player_system: PlayerManager,
  enemy_system: EnemyManager,
}




```

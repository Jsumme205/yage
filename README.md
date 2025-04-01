# YAGE
Yet Another Game Engine

This game engine is attempting to be a more data-oriented design.

for example,

one could write (simplified):
```rust

use yage::prelude::*;

struct Game {
  core: EngineCore,
  player_system: PlayerManager,
  enemy_system: EnemyManager,
}


```

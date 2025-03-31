use std::sync::Arc;

use crate::assets::AssetLoader;
use crate::system::Systems;
use crate::window::Window;
use yage_core::events::{self, DerivedFromEvent, ListenerCallback, Manager, Sender};

pub struct Context<'a, S> {
    state: &'a mut S,
    window: &'a mut Window,
}

pub trait Engine<Inputs> {
    type GlobalState;
    type MainEvent;
    type Loader: AssetLoader;
    type SubSystems: Systems<Inputs>;

    fn init(subsystems: Self::SubSystems, loader: Self::Loader, state: Self::GlobalState) -> Self;

    fn update(&mut self);

    fn run(&mut self) {
        while !self.should_exit() {
            self.update();
        }
    }

    fn should_exit(&self) -> bool;

    fn with_context<F>(&mut self, f: F) -> crate::Result<()>
    where
        F: FnOnce(&mut Context<'a, Self::GlobalState>) -> crate::Result<()>;

    fn attach<E>(&mut self, event: E) -> Arc<Manager<E, Self::MainEvent>>
    where
        E: ListenerCallback<Event: DerivedFromEvent<Self::MainEvent>>;
}

pub struct EngineCore<L> {
    loader: L,
    window: Window,
}

/// an example layout for a game engine
/// this is pretty bare bones, and won't even work but it's a good example for me to base my work off of
mod example {
    use std::task::Waker;

    use super::*;

    struct DummyLoader;

    impl AssetLoader for DummyLoader {
        fn load<P: AsRef<str>>(&mut self, path: P) -> crate::Result<crate::assets::Asset> {
            todo!()
        }

        fn load_as_bytes<P: AsRef<str>>(&mut self, path: P) -> crate::Result<Vec<u8>> {
            Ok(Vec::new())
        }
    }

    enum WeaponKind {
        Sword,
        Mace,
    }

    yage_system::component_data! {
        pub struct Player {
            weapon -> WeaponKind,
            health -> u8,
            position -> yage_system::Vec3,
        }
    }

    yage_system::component_data! {
        pub struct Enemy {
            weapon -> WeaponKind,
            sees_player -> bool,
            health -> u8,
            velocity -> yage_system::Vec3,
        }
    }

    struct ExampleEngine {
        players: Player,
        enemies: Enemy,
        listeners: Vec<Waker>,
        event_senders: Vec<Sender<Self::MainEvent>>,
        core: EngineCore<DummyLoader>,
    }

    impl Engine<(PlayerInstance, EnemyInstance)> for ExampleEngine {
        type GlobalState = ();
        type Loader = DummyLoader;
        type MainEvent = ();
        type SubSystems = (Player, Enemy);

        fn init(
            subsystems: Self::SubSystems,
            loader: Self::Loader,
            state: Self::GlobalState,
        ) -> Self {
            let (players, enemies) = subsystems;
            Self {
                players,
                enemies,
                listeners: Vec::new(),
                event_senders: Vec::new(),
                core: EngineCore {
                    loader,
                    window: todo!(),
                },
            }
        }

        fn run(&mut self) {}

        fn attach<E>(&mut self, event: E) -> Arc<Manager<E, Self::MainEvent>>
        where
            E: ListenerCallback<Event: DerivedFromEvent<Self::MainEvent>>,
        {
            let (tx, rx) = events::channel();
            let manager = Arc::new(Manager::new(event, rx));
            self.event_senders.push(tx);
            self.listeners.push(Waker::noop().clone());
            todo!()
        }

        fn update(&mut self) {
            //self.event_senders.iter_mut().for_each(||);
        }

        fn should_exit(&self) -> bool {
            false
        }

        fn with_context<F>(&mut self, f: F) -> crate::Result<()>
        where
            F: FnOnce(&mut Context<'a, Self::GlobalState>) -> crate::Result<()>,
        {
            f(&mut Context {
                state: &mut (),
                window: &mut self.core.window,
            })
        }
    }
}

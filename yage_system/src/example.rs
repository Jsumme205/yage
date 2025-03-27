use std::{alloc::Layout, f32::consts::PI, marker::PhantomData, ptr::NonNull};

use crate::{System, utility_structs::*};
use yage_core::allocator_api::Allocator;
use yage_util::container_trait::{Container, Represents};

pub struct Players {
    header: Header<PlayersLayout>,
    entities: ThinSlice<Entity>,
    mass: ThinSlice<f32>,
    position: ThinSlice<Vec3>,
    velocity: ThinSlice<Vec3>,
}

struct Player {
    entity: Entity,
    mass: f32,
    position: Vec3,
    velocity: Vec3,
}

struct PlayerIter<'a> {
    players: &'a Players,
    idx: u32,
    _marker: PhantomData<&'a [Player]>,
}

impl<'a> Represents<&'a Player> for PlayerRef<'a> {}
impl<'a> Represents<&'a mut Player> for PlayerMut<'a> {}

impl<'a> Iterator for PlayerIter<'a> {
    type Item = PlayerRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

struct PlayerIterMut<'a> {
    players: &'a mut Players,
    idx: u32,
    _marker: PhantomData<&'a mut [Player]>,
}

impl<'a> Iterator for PlayerIterMut<'a> {
    type Item = PlayerMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[derive(Clone, Copy)]
struct PlayerRef<'a> {
    entity: &'a Entity,
    mass: &'a f32,
    position: &'a Vec3,
    velocity: &'a Vec3,
}

struct PlayerMut<'a> {
    entity: &'a mut f32,
    position: &'a mut Vec3,
    velocity: &'a mut Vec3,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PlayersLayout {
    mass_offset: usize,
    position_offset: usize,
    velocity_offset: usize,
    align: usize,
    size: usize,
}

macro_rules! write_layout {
    ($place:expr, $val:expr, $off:expr) => {
        $place = $off;
        $off += $val
    };
}

impl PlayersLayout {
    /// I know that technically I could probably optimize for less code, but right now
    /// this is supposed to represent a macro output
    const fn eval_player_layout(number: u32) -> PlayersLayout {
        let number = number as usize;
        let align = core::mem::align_of::<Players>();
        let entity_size = size_padded_to_align::<Entity>(number, align);
        let mass_size = size_padded_to_align::<f32>(number, align);
        let position_size = size_padded_to_align::<Vec3>(number, align);
        let velocity_size = position_size;

        let mut layout = unsafe { core::mem::zeroed::<PlayersLayout>() };
        let mut offset = entity_size;
        layout.align = align;

        write_layout!(layout.mass_offset, mass_size, offset);
        write_layout!(layout.position_offset, position_size, offset);
        write_layout!(layout.velocity_offset, velocity_size, offset);

        layout.size = offset;
        layout
    }

    const fn layout_(&self) -> Layout {
        // SAFETY: these both came from `eval_player_layout`
        unsafe { Layout::from_size_align_unchecked(self.size, self.align) }
    }
}

impl GetLayout for PlayersLayout {
    fn layout(&self) -> Layout {
        self.layout_()
    }
}

macro_rules! write_ptr {
    ($place:expr, $len:expr, $align:expr, $val:expr) => {{
        let ptr = $place.byte_add($len * $align);
        ptr.write_unaligned($val);
    }};
}

macro_rules! ref_ptr {
    ($place:expr, $len:expr, $align:expr) => {{
        let ptr = $place.byte_add($len * $align);
        unsafe { ptr.as_ref() }
    }};
}

macro_rules! mut_ptr {
    ($place:expr, $len:expr, $align:expr) => {
        let ptr = $place.byte_add($len * $align);
        unsafe { ptr.as_mut() }
    };
}

const fn size_padded_to_align<T>(number: usize, align: usize) -> usize {
    unsafe {
        Layout::from_size_align_unchecked(core::mem::size_of::<T>() * number, align)
            .pad_to_align()
            .size()
    }
}

impl Players {
    fn allocate<A>(number: u32, allocator: &mut A) -> Result<Self, A::Error>
    where
        A: Allocator,
    {
        // unfortunately, if the number isn't constant, we have to leak a heap allocation
        // TODO: find a way around this?
        let layout: &'static PlayersLayout =
            Box::leak(Box::new(PlayersLayout::eval_player_layout(number)));
        let header = Header::new(false, number, layout);
        Self::__allocate_inner(allocator, header)
    }

    /// this version avoids an extra heap allocation; if you have a fixed capacity,
    /// use this
    fn allocate_array_layout<A, const N: usize>(allocator: &mut A) -> Result<Self, A::Error>
    where
        A: Allocator,
    {
        let header = Header::new(
            true,
            N as _,
            const { &PlayersLayout::eval_player_layout(N as _) },
        );
        Self::__allocate_inner(allocator, header)
    }

    fn __allocate_inner<A>(
        allocator: &mut A,
        header: Header<PlayersLayout>,
    ) -> Result<Self, A::Error>
    where
        A: Allocator,
    {
        let ptr = allocator.allocate(header.layout())?;
        let raw = ptr.as_ptr();
        let layout = header.layout;
        let this = unsafe {
            Self {
                header,
                entities: ThinSlice::from_raw(raw.cast()),
                mass: ThinSlice::from_raw(raw.add(layout.mass_offset).cast()),
                position: ThinSlice::from_raw(raw.add(layout.position_offset).cast()),
                velocity: ThinSlice::from_raw(raw.add(layout.velocity_offset).cast()),
            }
        };
        Ok(this)
    }

    fn transform<I, IF, CF>(collection_fn: CF, iter_fn: I) -> impl System<Player, Collection = Self>
    where
        Option<IF>: From<I>,
        IF: FnMut(PlayerMut<'_>),
        CF: FnMut(&mut Self),
    {
        struct SystemTransform<IF, CF>(Option<IF>, CF);

        impl<IF, CF> System<Player> for SystemTransform<IF, CF>
        where
            IF: FnMut(PlayerMut<'_>),
            CF: FnMut(&mut Players),
        {
            type Collection = Players;

            fn run_system(&mut self, collection: &mut Self::Collection) {
                (self.1)(collection)
            }

            fn consume_iter(&mut self, iter: <Self::Collection as Container<Player>>::Mutable<'_>) {
                for item in iter {
                    if let Some(ref mut f) = self.0 {
                        (f)(item)
                    }
                }
            }
        }

        SystemTransform(Option::from(iter_fn), collection_fn)
    }

    const fn push(&mut self, player: Player) -> yage_core::Result<Entity> {
        // technically we could also grow, but right now we're rolling a constant size
        if self.header.is_full() {
            return Err(yage_core::errors::Error::new(
                yage_core::ErrorKind::PushError,
            ));
        }

        let Player {
            entity,
            mass,
            position,
            velocity,
        } = player;

        let id = entity.id;

        unsafe {
            let len = self.header.len as usize;
            let align = self.header.layout.align;
            write_ptr!(self.entities.ptr.as_ptr(), len, align, entity);
            write_ptr!(self.mass.ptr.as_ptr(), len, align, mass);
            write_ptr!(self.position.ptr.as_ptr(), len, align, position);
            write_ptr!(self.velocity.ptr.as_ptr(), len, align, velocity);
        }
        self.header.len += 1;

        Ok(Entity { id })
    }

    //fn spawn(&mut self, f: impl FnOnce(Entity) -> Player)
}

impl Container<Player> for Players {
    type Iterator<'a> = PlayerIter<'a>;
    type Mutable<'a> = PlayerIterMut<'a>;

    fn iterator(&self) -> Self::Iterator<'_> {
        PlayerIter {
            players: self,
            idx: 0,
            _marker: PhantomData,
        }
    }

    fn mutable_iterator(&mut self) -> Self::Mutable<'_> {
        PlayerIterMut {
            players: self,
            idx: 0,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::NonNull;

    /// extremely unsafe, only used to test conditions
    /// like NO. it has zero checks at all
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
    struct TrackingAlloc {
        bytes: usize,
    }

    unsafe impl Allocator for TrackingAlloc {
        type Error = ();

        fn allocate(
            &mut self,
            layout: std::alloc::Layout,
        ) -> Result<std::ptr::NonNull<u8>, Self::Error> {
            self.bytes += layout.size();
            unsafe { Ok(NonNull::new_unchecked(std::alloc::alloc(layout))) }
        }

        fn deallocate(
            &mut self,
            ptr: *mut u8,
            layout: std::alloc::Layout,
        ) -> Result<(), Self::Error> {
            unsafe { std::alloc::dealloc(ptr, layout) };
            Ok(())
        }

        fn reallocate(
            &mut self,
            old_ptr: *mut u8,
            old_layout: std::alloc::Layout,
            new_layout: std::alloc::Layout,
        ) -> Result<NonNull<u8>, Self::Error> {
            Err(())
        }
    }

    #[test]
    fn test_player_allocate() {
        let mut alloc = TrackingAlloc::default();
        let mut players = Players::allocate_array_layout::<_, 10>(&mut alloc).unwrap();
        let player = Player {
            entity: Entity { id: 0 },
            mass: 3.0,
            position: Vec3(0.0, 0.0, 0.0),
            velocity: Vec3(0.0, 0.0, 0.0),
        };
        let _ = players.push(player);
        dbg!(alloc);
    }

    #[test]
    fn test_transform() {
        //let system = Players::transform(|c| (), None);
    }
}

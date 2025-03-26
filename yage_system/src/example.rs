use crate::utility_structs::*;
use core::marker::{PhantomData, PhantomPinned};
use yage_core::allocator_api::Allocator;

pub struct Players {
    header: Header,
    entities: ThinSlice<Entity>,
    weight: ThinSlice<f32>,
    position: ThinSlice<Vec3>,
    velocity: ThinSlice<Vec3>,
    _pin: PhantomPinned,
}

struct PlayersLayout {}

impl Players {
    const DANGLING: Self = Self {
        header: Header::DANGLING,
        entities: ThinSlice::DANGLING,
        weight: ThinSlice::DANGLING,
        position: ThinSlice::DANGLING,
        velocity: ThinSlice::DANGLING,
        _pin: PhantomPinned,
    };

    fn allocate<A>(number: u32, allocator: &mut A) -> Result<Self, A::Error>
    where
        A: Allocator,
    {
    }
}

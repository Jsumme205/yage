#[derive(Clone, Copy)]
enum WeaponKind {
    Sword,
    Mace,
    Spear,
}

crate::component_data! {
    pub struct Players {
        weapon -> WeaponKind,
        health -> u8,
        position -> crate::Vec3,
    }
}

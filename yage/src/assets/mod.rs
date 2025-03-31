pub trait AssetLoader {
    fn load_as_bytes<P: AsRef<str>>(&mut self, path: P) -> crate::Result<Vec<u8>>;
    fn load<P: AsRef<str>>(&mut self, path: P) -> crate::Result<Asset>;
}

pub struct Asset {
    raw: Vec<u8>,
    kind: AssetKind,
}

enum AssetKind {}

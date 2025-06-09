use crate::cache::AssetCache;
use std::fs as file;
use std::io;
use std::sync::Arc;
use yage_core::asset::CowHandle;
use yage_core::asset::Loader;
use yage_core::states::new::SearchPaths;

pub struct FileSystem {
    search_paths: Vec<&'static str>,
    cache: ,
}

struct FsFut<'a> {
    this: &'a FileSystem,
}

impl<'a> Future for FsFut<'a> {
    type Output = io::Result<CowHandle<'a, AssetCache>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

impl Loader<SearchPaths, &str, AssetCache> for FileSystem {
    type Error = io::Error;
    type InitFuture = std::future::Ready<io::Result<Self>>;
    type LoadFuture<'a> = FsFut<'a>;

    fn init(init: SearchPaths) -> Self::InitFuture
    where
        Self: Sized,
    {
        std::future::ready(Ok(Self { search_paths: init.paths }))
    }

    fn load(&self, name: &str) -> Self::LoadFuture<'_> {
        FsFut { this: self }
    }
}

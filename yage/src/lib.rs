

pub mod fs;
pub mod cache;
pub mod net;

pub type New = yage_core::prelude::New<
  fs::FileSystem, 
  net::Network, 
  std::io::Error, 
  cache::AssetCache,
  std::future::Ready<fs::FileSystem>,
  std::future::Ready<net::Network>
>;

pub type Loading<L> = yage_core::prelude::Loading<
  L,
  fs::FileSystem,
  net::Network,
  cache::AssetCache
>;

pub use yage_core::App;

fn test() {
  let mut app = App::<New>::new();
}


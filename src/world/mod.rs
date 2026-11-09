pub mod noise;

mod collider;
mod helpers;
mod world;

pub use world::World;

pub trait Generator {
    fn generate(&self) -> (Vec<u128>, Vec<u8>);
}

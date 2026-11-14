use super::{Element, builder::UiBuilder};
use crate::{common::Buildable, engine::Record, texture::Atlas};
use std::collections::HashMap;

pub struct Ui {
    pub(super) elements: HashMap<String, Box<dyn Element>>,
    pub(super) textures: Atlas,
}

impl Buildable for Ui {
    type Builder = UiBuilder;
}

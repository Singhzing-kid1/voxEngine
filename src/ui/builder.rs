use super::Element;
use crate::{
    common::Builder,
    engine::Engine,
    texture::{Atlas, builder::AtlasBuilder},
    ui::Ui,
};
use rapier3d::glamx::{Mat2, Vec2, mat2, vec2};
use std::{collections::HashMap, path::PathBuf};
use vulkano::format::Format;

#[derive(Default)]
pub struct UiBuilder {
    elements: Option<HashMap<String, &mut Box<dyn Element>>>,
    textures: Option<AtlasBuilder>,
}

impl UiBuilder {
    pub fn add_element(
        mut self,
        element: &mut Box<dyn Element>,
        origin: Vec2,
        size: Vec2,
        texture: PathBuf,
    ) -> Self {
        let key = mat2(origin, origin + size);

        let builder = self.textures.take().unwrap_or_else(|| {
            AtlasBuilder::default()
                .set_format(Format::R8G8B8A8_UNORM)
                .set_lod_bias(0f32)
        });

        let texture_id = builder.layers.unwrap();

        element.as_mut().set_texture(texture_id);

        self.textures = Some(builder.add_image(texture));

        self.elements
            .get_or_insert_default()
            .entry(format!("{key}"))
            .or_insert(element);

        self
    }
}

impl Builder for UiBuilder {
    type Output = Ui;
    type Error = &'static str;

    fn build(self, engine: &mut Engine) -> Result<Self::Output, Self::Error> {
        let textures = self.textures.unwrap_or_default().build(engine).unwrap();

        Ok(Ui {
            elements: self.elements.unwrap(),
            textures,
        })
    }
}

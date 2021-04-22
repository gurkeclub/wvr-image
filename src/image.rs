use std::fs::File;
use std::io::BufReader;

use anyhow::{Context, Result};
use image::imageops::FilterType;
use image::ImageFormat;

use wvr_data::DataHolder;
use wvr_data::InputProvider;

pub struct PictureProvider {
    name: String,
    resolution: (u32, u32),
    texture_data: Vec<u8>,
    invalidated: bool,
}

impl PictureProvider {
    pub fn new(path: &str, name: String, resolution: (usize, usize)) -> Result<Self> {
        let format = ImageFormat::from_path(path)
            .context("Failed to deduce image format from image file path")?;
        let image = image::load(
            BufReader::new(File::open(path).context("Failed to open image file:")?),
            format,
        )
        .context("Failed to load image file")?
        .resize(
            resolution.0 as u32,
            resolution.1 as u32,
            FilterType::Lanczos3,
        )
        .to_rgb8();

        let image = image::imageops::flip_vertical(&image);

        Ok(Self {
            name,
            resolution: image.dimensions(),
            texture_data: image.to_vec(),
            invalidated: false,
        })
    }
}

impl InputProvider for PictureProvider {
    fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }
    
    fn provides(&self) -> Vec<String> {
        vec![self.name.clone()]
    }

    fn get(&mut self, uniform_name: &str, invalidate: bool) -> Option<DataHolder> {
        if self.invalidated {
            return None;
        }

        if uniform_name == self.name {
            if invalidate {
                self.invalidated = true;
            }

            Some(DataHolder::Texture((
                self.resolution,
                self.texture_data.to_vec(),
            )))
        } else {
            None
        }
    }
}

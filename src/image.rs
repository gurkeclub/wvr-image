use std::fs::File;
use std::io::BufReader;

use anyhow::{Context, Result};
use image::imageops::FilterType;
use image::DynamicImage;
use image::ImageFormat;

use wvr_data::DataHolder;
use wvr_data::InputProvider;

pub struct PictureProvider {
    name: String,
    resolution: (u32, u32),
    original_picture: DynamicImage,
    texture_data: Vec<u8>,
    invalidated: bool,
}

impl PictureProvider {
    pub fn new(path: &str, name: String, resolution: (usize, usize)) -> Result<Self> {
        let format = ImageFormat::from_path(path)
            .context("Failed to deduce image format from image file path")?;

        let original_picture = image::load(
            BufReader::new(File::open(path).context("Failed to open image file:")?),
            format,
        )
        .context("Failed to load image file")?;

        let image = original_picture
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
            original_picture,
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

    fn set_property(&mut self, property: &str, value: &DataHolder) {
        match (property, value) {
            ("resolution", DataHolder::Int2(new_resolution)) => {
                let new_resolution = (new_resolution[0] as u32, new_resolution[1] as u32);
                if new_resolution != self.resolution {
                    self.resolution = new_resolution;

                    let image = self
                        .original_picture
                        .resize(
                            self.resolution.0 as u32,
                            self.resolution.1 as u32,
                            FilterType::Lanczos3,
                        )
                        .to_rgb8();

                    self.resolution = image.dimensions();
                    self.texture_data = image::imageops::flip_vertical(&image).to_vec();
                    self.invalidated = false;
                }
            }
            ("width", DataHolder::Int(new_width)) => {
                let new_resolution = (*new_width as u32, self.resolution.1);
                if new_resolution != self.resolution {
                    self.resolution = new_resolution;

                    let image = self
                        .original_picture
                        .resize(
                            self.resolution.0 as u32,
                            self.resolution.1 as u32,
                            FilterType::Lanczos3,
                        )
                        .to_rgb8();

                    self.resolution = image.dimensions();
                    self.texture_data = image::imageops::flip_vertical(&image).to_vec();
                    self.invalidated = false;
                }
            }

            ("height", DataHolder::Int(new_height)) => {
                let new_resolution = (self.resolution.0, *new_height as u32);
                if new_resolution != self.resolution {
                    self.resolution = new_resolution;

                    let image = self
                        .original_picture
                        .resize(
                            self.resolution.0 as u32,
                            self.resolution.1 as u32,
                            FilterType::Lanczos3,
                        )
                        .to_rgb8();

                    self.resolution = image.dimensions();
                    self.texture_data = image::imageops::flip_vertical(&image).to_vec();
                    self.invalidated = false;
                }
            }
            ("path", DataHolder::String(new_path)) => {
                let format = ImageFormat::from_path(new_path)
                    .context("Failed to deduce image format from image file path")
                    .unwrap();

                self.original_picture = image::load(
                    BufReader::new(
                        File::open(new_path)
                            .context("Failed to open image file:")
                            .unwrap(),
                    ),
                    format,
                )
                .context("Failed to load image file")
                .unwrap();

                let image = self
                    .original_picture
                    .resize(
                        self.resolution.0 as u32,
                        self.resolution.1 as u32,
                        FilterType::Lanczos3,
                    )
                    .to_rgb8();

                self.resolution = image.dimensions();
                self.texture_data = image::imageops::flip_vertical(&image).to_vec();
                self.invalidated = false;
            }
            _ => eprintln!("Set_property unimplemented for {:}", property),
        }
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

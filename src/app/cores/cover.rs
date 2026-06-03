use crate::app::cores::files::file_finder;

use image::error::ImageError;
use image::{GenericImageView, ImageReader};
use std::path::Path;

pub fn square_crop_to_bytes(path: &Path) -> Result<Vec<u8>, ImageError> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?;
    let (width, height) = img.dimensions();
    let mut buf = Vec::new();

    let final_img = if width != height {
        log::info!("crop report: start crop");
        let side = width.min(height);
        let x = (width - side) / 2;
        let y = (height - side) / 2;
        img.crop_imm(x, y, side, side)
    } else {
        log::info!("crop report: skipp crop");
        img
    };

    final_img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

    std::fs::remove_file(path)?;

    Ok(buf)
}

pub fn to_png_bytes(path: &Path) -> Result<Vec<u8>, ImageError> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?;
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;
    std::fs::remove_file(path)?;
    Ok(buf)
}

pub fn embed(
    crop: bool,
    musicfile: &Path,
    directory: &str,
    filename: &str,
    playlist: &Option<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let single_cover = match file_finder(directory, filename, &["webp"]) {
        Some(single_cover) => single_cover,
        None => return Err("No Cover of Music Found".into()),
    };
    if let Some(cover) = playlist {
        embed_img_internal(cover, musicfile)?;
        std::fs::remove_file(single_cover)?;
    } else {
        let cover = if crop {
            square_crop_to_bytes(&single_cover)?
        } else {
            to_png_bytes(&single_cover)?
        };
        embed_img_internal(&cover, musicfile)?;
    }
    Ok(())
}

use lofty::config::WriteOptions;
use lofty::picture::{Picture, PictureType};
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Tag;
use std::io::{BufReader, Cursor};

fn embed_img_internal(cover: &Vec<u8>, musicfile: &Path) -> Result<(), lofty::error::LoftyError> {
    let mut reader = BufReader::new(Cursor::new(cover));
    let mut tagged_file = Probe::open(musicfile)?.read()?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag_mut() {
                first_tag
            } else {
                let tag_type = tagged_file.primary_tag_type();

                log::error!("No tags found, creating a new tag of type `{tag_type:?}`");
                tagged_file.insert_tag(Tag::new(tag_type));

                tagged_file.primary_tag_mut().unwrap()
            }
        }
    };
    let mut picture = Picture::from_reader(&mut reader)?;
    picture.set_pic_type(PictureType::CoverFront);

    tag.push_picture(picture);

    if tag.save_to_path(musicfile, WriteOptions::default()).is_ok() {
        log::info!("Cover report: Embedded Success");
    } else {
        log::error!("Cover report: Embedded Fail");
    }
    Ok(())
}

use image::error::ImageError;
use image::{GenericImageView, ImageReader};
use std::fs;
use std::path::{Path, PathBuf};

fn square_crop_to_png(path: &Path) -> Result<(), ImageError> {
    let img = ImageReader::open(path)?.with_guessed_format()?.decode()?;
    let (width, height) = img.dimensions();
    let png_path = path.with_extension("png");

    let final_img = if width != height {
        println!("crop report: start crop");
        let side = width.min(height);
        let x = (width - side) / 2;
        let y = (height - side) / 2;
        let cropped = img.crop_imm(x, y, side, side);
        cropped
    } else {
        println!("crop report: skipp crop");
        img
    };

    final_img.save_with_format(&png_path, image::ImageFormat::Png)?;

    if path != &png_path {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

fn to_png(path: &Path) -> Result<(), ImageError> {
    if path.extension().and_then(|x| x.to_str()) != Some("png") {
        let img = ImageReader::open(path)?.with_guessed_format()?.decode()?;
        let png_path = path.with_extension("png");

        img.save_with_format(&png_path, image::ImageFormat::Png)?;
        if path != &png_path {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

pub fn embed(
    crop: bool,
    playlist_cover: bool,
    musicfile: &Path,
    directory: &str,
    filename: &str,
    playlist_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let single_cover = match image_finder(directory, filename, &["webp"]) {
        Some(single_cover) => single_cover,
        None => return Err("No Cover of Music Found".into()),
    };
    if let Some(name) = playlist_name
        && playlist_cover
    {
        match image_finder(directory, name, &["jpg", "jpeg", "png"]) {
            Some(raw_image) => {
                let png = raw_image.with_extension("png");
                println!("Cover report raw_image: {raw_image:?}");
                if crop {
                    square_crop_to_png(&raw_image)?
                } else {
                    to_png(&raw_image)?;
                }
                println!("Cover report png: {png:?}");
                embed_img_internal(&png, musicfile)?;
                std::fs::remove_file(single_cover)?;
            }
            None => return Err("No Cover of Playlist Found".into()),
        }
    } else {
        let png = single_cover.with_extension("png");
        if crop {
            square_crop_to_png(&single_cover)?
        } else {
            to_png(&single_cover)?;
        }
        embed_img_internal(&png, musicfile)?;
        std::fs::remove_file(png)?;
    }
    Ok(())
}

use lofty::config::WriteOptions;
use lofty::picture::{Picture, PictureType};
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Tag;
use std::fs::File;
use std::io::BufReader;

fn embed_img_internal(cover: &Path, musicfile: &Path) -> Result<(), lofty::error::LoftyError> {
    let f = File::open(cover)?;
    let mut reader = BufReader::new(f);
    let mut tagged_file = Probe::open(&musicfile)?.read()?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag_mut() {
                first_tag
            } else {
                let tag_type = tagged_file.primary_tag_type();

                eprintln!("WARN: No tags found, creating a new tag of type `{tag_type:?}`");
                tagged_file.insert_tag(Tag::new(tag_type));

                tagged_file.primary_tag_mut().unwrap()
            }
        }
    };
    let mut picture = Picture::from_reader(&mut reader)?;
    picture.set_pic_type(PictureType::CoverFront);

    tag.push_picture(picture);

    if tag.save_to_path(musicfile, WriteOptions::default()).is_ok() {
        println!("Cover report: Embedded Sucsess");
    } else {
        eprintln!("Cover report: Embedded Fail")
    }
    Ok(())
}

static VALID_FORMAT: &[&str] = &["jpg", "webp", "jpeg", "png"];

fn image_finder(directory: &str, filename: &str, matchs: &[&str]) -> Option<PathBuf> {
    let elements = fs::read_dir(&directory).ok()?;

    for item in elements {
        let path = item.ok()?.path();
        if path.is_file() {
            let ext = path.extension().and_then(|ext| ext.to_str())?;

            if matchs.contains(&ext) {
                let file = path.file_name().and_then(|name| name.to_str())?;
                if file.contains(filename) {
                    let good_file = Some(path);
                    return good_file;
                }
            }
        }
    }
    None
}

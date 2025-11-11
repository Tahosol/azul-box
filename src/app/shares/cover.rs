use image::error::ImageError;
use image::{GenericImageView, ImageReader};
use std::path::Path;

fn square_crop_to_png(path: &Path) -> Result<(), ImageError> {
    let img = ImageReader::open(path)?.decode()?;
    let (width, height) = img.dimensions();
    let png_path = path.with_extension("png");

    let final_img = if width != height {
        let side = width.min(height);
        let x = (width - side) / 2;
        let y = (height - side) / 2;
        let cropped = img.crop_imm(x, y, side, side);
        cropped
    } else {
        img
    };

    final_img.save_with_format(&png_path, image::ImageFormat::Png)?;

    if path != &png_path {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

fn to_png(path: &Path) -> Result<(), ImageError> {
    let img = ImageReader::open(path)?.decode()?;
    let png_path = path.with_extension("png");

    img.save_with_format(&png_path, image::ImageFormat::Png)?;
    if path != &png_path {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

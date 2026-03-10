use edge_segmentation::segment::*;

use image::{ImageReader, Rgb, RgbImage};
//use plotters::prelude::*;

const OUT_FILE_NAME: &str = "test.png";
const THRESHOLD: f32 = 0.5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = ImageReader::open("funhouse.jpg")?.decode()?;

    let sobel_x = img
        .filter3x3(&[
            -0.25,
            0.0,
            0.25,
            -0.5,
            0.0,
            0.5,
            -0.25,
            0.0,
            0.25,
        ])
        .into_rgb32f();
    let sobel_y = img
        .filter3x3(&[
            -0.25,
            -0.5,
            -0.25,
            0.0,
            0.0,
            0.0,
            0.25,
            0.5,
            0.25,
        ])
        .into_rgb32f();

    let mut new_img = RgbImage::new(img.width(), img.height());

    for ((x, y, &cx), &cy) in sobel_x.enumerate_pixels().zip(sobel_y.pixels()) {
        //println!("{} {}", x, y);
        let dif = f32::sqrt(
            cx.0[0] * cx.0[0]
                + cx.0[1] * cx.0[1]
                + cx.0[2] * cx.0[2]
                + cy.0[0] * cy.0[0]
                + cy.0[1] * cy.0[1]
                + cy.0[2] * cy.0[2],
        );
        //println!("{}", dif);
        if dif > THRESHOLD {
            new_img.put_pixel(x, y, Rgb([255, 255, 255]));
        } else {
            new_img.put_pixel(x, y, Rgb([0, 0, 0]));
        }
    }

    new_img.save(OUT_FILE_NAME)?;

    Ok(())
}

use edge_segmentation::{segment::*, segmentation::*};

use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgb, RgbImage};
//use plotters::prelude::*;

const OUT_FILE_NAME: &str = "edges.png";
const THRESHOLD: f32 = 0.2;

fn extract_sobel(img: &DynamicImage) -> Vec<bool> {
    let sobel_x = img
        .filter3x3(&[-0.25, 0.0, 0.25, -0.5, 0.0, 0.5, -0.25, 0.0, 0.25])
        .into_rgb32f();
    let sobel_y = img
        .filter3x3(&[-0.25, -0.5, -0.25, 0.0, 0.0, 0.0, 0.25, 0.5, 0.25])
        .into_rgb32f();

    let mut result = vec![false; (img.width() * img.height()) as usize];
    for ((x, y, &cx), &cy) in sobel_x.enumerate_pixels().zip(sobel_y.pixels()) {
        let sob = cx.0[0] * cx.0[0]
            + cx.0[1] * cx.0[1]
            + cx.0[2] * cx.0[2]
            + cy.0[0] * cy.0[0]
            + cy.0[1] * cy.0[1]
            + cy.0[2] * cy.0[2];
        //println!("{} {}", x, y);
        let dif = f32::sqrt(sob);
        result[(x + img.width() * y) as usize] = dif > THRESHOLD;
    }

    result
}

fn draw_segments(
    img: &DynamicImage,
    bool_img: &[bool],
    segments: &Vec<(Segment, usize)>,
    width: u32,
    height: u32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let get_col = |i: usize| {
        Rgb([
            ((113 * (i + 1)) % 256) as u8,
            ((127 * (i + 2)) % 256) as u8,
            ((131 * (i + 3)) % 256) as u8,
        ])
    };

    let mut blackboard = RgbImage::new(3 * width, height);

    for x in 0..width {
        for y in 0..height {
            blackboard.put_pixel(x, y, img.get_pixel(x, y).to_rgb());
            if bool_img[(x + width * y) as usize] {
                blackboard.put_pixel(width + x, y, Rgb([255, 255, 255]));
            } else {
                blackboard.put_pixel(width + x, y, Rgb([0, 0, 0]));
            }
        }
    }

    for (i, (segment, _)) in segments.iter().enumerate() {
        let pixels = segment.list_in_pixels(bool_img, width as usize);
        for px in pixels.iter() {
            if (px.0 as u32) < width && (px.1 as u32) < height {
                blackboard.put_pixel(2 * width + px.0 as u32, px.1 as u32, get_col(i));
            }
        }
    }

    blackboard
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = ImageReader::open("funhouse.jpg")?.decode()?;
    let sobel = extract_sobel(&img);
    let edges = segment_edges(&sobel, img.height() as usize, img.width() as usize, 32);
    println!("{}", edges.len());
    //println!("{:?}", edges);

    let new_img = draw_segments(&img, &sobel, &edges, img.width(), img.height());

    new_img.save(OUT_FILE_NAME)?;

    Ok(())
}

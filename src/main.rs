use std::time::Instant;

use edge_segmentation::{quadtree_idea::*, segment::*, segmentation::*};

//use std::time::Instant;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgb, RgbImage};
//use plotters::prelude::*;

const OUT_FILE_NAME: &str = "edges.png";
const THRESHOLD: f32 = 0.15;

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

fn extract_sobel_cfar(img: &DynamicImage) -> Vec<(f32, f32)> {
    let sobel_x = img
        .filter3x3(&[-0.25, 0.0, 0.25, -0.5, 0.0, 0.5, -0.25, 0.0, 0.25])
        .into_rgb32f();
    let sobel_y = img
        .filter3x3(&[-0.25, -0.5, -0.25, 0.0, 0.0, 0.0, 0.25, 0.5, 0.25])
        .into_rgb32f();

    let mut sob_array = vec![0f32; (img.width() * img.height()) as usize];
    for ((x, y, &cx), &cy) in sobel_x.enumerate_pixels().zip(sobel_y.pixels()) {
        sob_array[(x + img.width() * y) as usize] = cx.0[0] * cx.0[0]
                                                  + cx.0[1] * cx.0[1]
                                                  + cx.0[2] * cx.0[2]
                                                  + cy.0[0] * cy.0[0]
                                                  + cy.0[1] * cy.0[1]
                                                  + cy.0[2] * cy.0[2];
    }

    let mut result = vec![];
    let w = img.width() as usize;
    for i in 3..(w - 3) {
        for j in 3.. ((img.height() - 3) as usize) {
            let cfar_sample = vec![
                // first row
                sob_array[i - 3 + w*(j - 3)],
                sob_array[i - 2 + w*(j - 3)],
                sob_array[i - 1 + w*(j - 3)],
                sob_array[i + w*(j - 3)],
                sob_array[i + 1 + w*(j - 3)],
                sob_array[i + 2 + w*(j - 3)],
                sob_array[i + 3 + w*(j - 3)],
                // second row
                sob_array[i - 3 + w*(j - 2)],
                sob_array[i - 2 + w*(j - 2)],
                sob_array[i - 1 + w*(j - 2)],
                sob_array[i + w*(j - 2)],
                sob_array[i + 1 + w*(j - 2)],
                sob_array[i + 2 + w*(j - 2)],
                sob_array[i + 3 + w*(j - 2)],
                // third row
                sob_array[i - 3 + w*(j - 1)],
                sob_array[i - 2 + w*(j - 1)],
                sob_array[i + 2 + w*(j - 1)],
                sob_array[i + 3 + w*(j - 1)],
                // fourth row
                sob_array[i - 3 + w*j],
                sob_array[i - 2 + w*j],
                sob_array[i + 2 + w*j],
                sob_array[i + 3 + w*j],
                // fifth row
                sob_array[i - 3 + w*(j + 1)],
                sob_array[i - 2 + w*(j + 1)],
                sob_array[i + 2 + w*(j + 1)],
                sob_array[i + 3 + w*(j + 1)],
                // sixth row
                sob_array[i - 3 + w*(j + 2)],
                sob_array[i - 2 + w*(j + 2)],
                sob_array[i - 1 + w*(j + 2)],
                sob_array[i + w*(j + 2)],
                sob_array[i + 1 + w*(j + 2)],
                sob_array[i + 2 + w*(j + 2)],
                sob_array[i + 3 + w*(j + 2)],
                // seventh row
                sob_array[i - 3 + w*(j + 3)],
                sob_array[i - 2 + w*(j + 3)],
                sob_array[i - 1 + w*(j + 3)],
                sob_array[i + w*(j + 3)],
                sob_array[i + 1 + w*(j + 3)],
                sob_array[i + 2 + w*(j + 3)],
                sob_array[i + 3 + w*(j + 3)],
            ];
            let thresh = quickselect(cfar_sample, 30);
            if sob_array[i + w*j] > thresh && sob_array[i + w*j] > THRESHOLD {
                result.push((i as f32, j as f32));
            }
        }
    }

    result
}

fn extract_sobel_pointcloud(img: &DynamicImage) -> Vec<(f32, f32)> {
    let sobel_x = img
        .filter3x3(&[-0.25, 0.0, 0.25, -0.5, 0.0, 0.5, -0.25, 0.0, 0.25])
        .into_rgb32f();
    let sobel_y = img
        .filter3x3(&[-0.25, -0.5, -0.25, 0.0, 0.0, 0.0, 0.25, 0.5, 0.25])
        .into_rgb32f();

    let mut result = vec![];
    for ((x, y, &cx), &cy) in sobel_x.enumerate_pixels().zip(sobel_y.pixels()) {
        let sob = cx.0[0] * cx.0[0]
            + cx.0[1] * cx.0[1]
            + cx.0[2] * cx.0[2]
            + cy.0[0] * cy.0[0]
            + cy.0[1] * cy.0[1]
            + cy.0[2] * cy.0[2];
        //println!("{} {}", x, y);
        let dif = f32::sqrt(sob);
        if dif > THRESHOLD {
            result.push((x as f32, y as f32));
        }
    }

    result
}

fn draw_segments(
    img: &DynamicImage,
    sobel_pointcloud: &Vec<(f32, f32)>,
    segments: &Vec<Segment>,
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
        }
    }

    for pixel in sobel_pointcloud.iter() {
        blackboard.put_pixel(width + (pixel.0 as u32), pixel.1 as u32, Rgb([255, 255, 255]));
    }

    for (i, segment) in segments.iter().enumerate() {
        let pixels = segment.list_all_pixels(width as usize);
        for px in pixels.iter() {
            if (px.0 as u32) < width && (px.1 as u32) < height {
                blackboard.put_pixel(2*width + px.0 as u32, px.1 as u32, get_col(i));
            }
        }
    }

    blackboard
}

fn draw_points(
    img: &DynamicImage,
    points: &Vec<Vec<(f32, f32)>>,
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

    let mut blackboard = RgbImage::new(2 * width, height);

    /*
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
    */

    for (i, pixels) in points.iter().enumerate() {
        //let pixels = segment.list_in_pixels(bool_img, width as usize);
        for px in pixels.iter() {
            if (px.0 as u32) < width && (px.1 as u32) < height {
                blackboard.put_pixel(width + (px.0 as u32), px.1 as u32, get_col(i));
            }
        }
    }

    blackboard
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = ImageReader::open("funhouse.jpg")?.decode()?;
    let sobel = extract_sobel_cfar(&img);
    //println!("{}", edges.len());
    //println!("{:?}", edges);
    let now = Instant::now();
    let quadtree_pts = quadtree_cluster(&sobel, 6);
    let edges = detect_edges(&quadtree_pts);
    let elapsed = now.elapsed();
    println!("{} edges detected in {} milliseconds", edges.len(), elapsed.as_millis());

    let new_img = draw_segments(&img, &sobel, &edges, img.width(), img.height());
    //let new_img = draw_points(&img, &quadtree_pts, img.width(), img.height());

    new_img.save(OUT_FILE_NAME)?;

    Ok(())
}

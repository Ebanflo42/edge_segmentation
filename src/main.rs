use std::time::Instant;

use edge_segmentation::{quadtree_idea::*, segment::*, segmentation::*};

//use std::time::Instant;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgb, RgbImage};
//use plotters::prelude::*;

const OUT_FILE_NAME: &str = "trees_test.png";
const IN_FILE_NAME: &str = "trees.jpg";
const THRESHOLD: f32 = 0.15;
const QUANTILE: f32 = 0.95;
const CFAR_QUANTILE: f32 = 0.9;

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

fn extract_sobel_cfar(img: &DynamicImage) -> (Vec<(f32, f32)>, Vec<(f32, f32)>) {
    let sobel_x = img
        .filter3x3(&[-0.25, 0.0, 0.25, -0.5, 0.0, 0.5, -0.25, 0.0, 0.25])
        .into_rgb32f();
    let sobel_y = img
        .filter3x3(&[-0.25, -0.5, -0.25, 0.0, 0.0, 0.0, 0.25, 0.5, 0.25])
        .into_rgb32f();

    let mut sob_array_x = vec![0f32; (img.width() * img.height()) as usize];
    let mut sob_array_y = vec![0f32; (img.width() * img.height()) as usize];
    for ((x, y, &cx), &cy) in sobel_x.enumerate_pixels().zip(sobel_y.pixels()) {
        sob_array_x[(x + img.width() * y) as usize] =
            cx.0[0] * cx.0[0] + cx.0[1] * cx.0[1] + cx.0[2] * cx.0[2];
        sob_array_y[(x + img.width() * y) as usize] =
            cy.0[0] * cy.0[0] + cy.0[1] * cy.0[1] + cy.0[2] * cy.0[2];
    }

    let threshold_x = quickselect(
        sob_array_x.clone(),
        (QUANTILE * (sob_array_x.len() as f32)) as usize,
    );
    let threshold_y = quickselect(
        sob_array_x.clone(),
        (QUANTILE * (sob_array_y.len() as f32)) as usize,
    );


    let mut result_x = vec![];
    let mut result_y = vec![];
    let w = img.width() as usize;
    for i in 3..(w - 3) {
        for j in 3..((img.height() - 3) as usize) {
            let cfar_sample_x = vec![
                // first row
                sob_array_x[i - 3 + w * (j - 3)],
                sob_array_x[i - 2 + w * (j - 3)],
                sob_array_x[i - 1 + w * (j - 3)],
                sob_array_x[i + w * (j - 3)],
                sob_array_x[i + 1 + w * (j - 3)],
                sob_array_x[i + 2 + w * (j - 3)],
                sob_array_x[i + 3 + w * (j - 3)],
                // second row
                sob_array_x[i - 3 + w * (j - 2)],
                sob_array_x[i - 2 + w * (j - 2)],
                sob_array_x[i - 1 + w * (j - 2)],
                sob_array_x[i + w * (j - 2)],
                sob_array_x[i + 1 + w * (j - 2)],
                sob_array_x[i + 2 + w * (j - 2)],
                sob_array_x[i + 3 + w * (j - 2)],
                // third row
                sob_array_x[i - 3 + w * (j - 1)],
                sob_array_x[i - 2 + w * (j - 1)],
                sob_array_x[i + 2 + w * (j - 1)],
                sob_array_x[i + 3 + w * (j - 1)],
                // fourth row
                sob_array_x[i - 3 + w * j],
                sob_array_x[i - 2 + w * j],
                sob_array_x[i + 2 + w * j],
                sob_array_x[i + 3 + w * j],
                // fifth row
                sob_array_x[i - 3 + w * (j + 1)],
                sob_array_x[i - 2 + w * (j + 1)],
                sob_array_x[i + 2 + w * (j + 1)],
                sob_array_x[i + 3 + w * (j + 1)],
                // sixth row
                sob_array_x[i - 3 + w * (j + 2)],
                sob_array_x[i - 2 + w * (j + 2)],
                sob_array_x[i - 1 + w * (j + 2)],
                sob_array_x[i + w * (j + 2)],
                sob_array_x[i + 1 + w * (j + 2)],
                sob_array_x[i + 2 + w * (j + 2)],
                sob_array_x[i + 3 + w * (j + 2)],
                // seventh row
                sob_array_x[i - 3 + w * (j + 3)],
                sob_array_x[i - 2 + w * (j + 3)],
                sob_array_x[i - 1 + w * (j + 3)],
                sob_array_x[i + w * (j + 3)],
                sob_array_x[i + 1 + w * (j + 3)],
                sob_array_x[i + 2 + w * (j + 3)],
                sob_array_x[i + 3 + w * (j + 3)],
            ];
            let cfar_sample_y = vec![
                // first row
                sob_array_y[i - 3 + w * (j - 3)],
                sob_array_y[i - 2 + w * (j - 3)],
                sob_array_y[i - 1 + w * (j - 3)],
                sob_array_y[i + w * (j - 3)],
                sob_array_y[i + 1 + w * (j - 3)],
                sob_array_y[i + 2 + w * (j - 3)],
                sob_array_y[i + 3 + w * (j - 3)],
                // second row
                sob_array_y[i - 3 + w * (j - 2)],
                sob_array_y[i - 2 + w * (j - 2)],
                sob_array_y[i - 1 + w * (j - 2)],
                sob_array_y[i + w * (j - 2)],
                sob_array_y[i + 1 + w * (j - 2)],
                sob_array_y[i + 2 + w * (j - 2)],
                sob_array_y[i + 3 + w * (j - 2)],
                // third row
                sob_array_y[i - 3 + w * (j - 1)],
                sob_array_y[i - 2 + w * (j - 1)],
                sob_array_y[i + 2 + w * (j - 1)],
                sob_array_y[i + 3 + w * (j - 1)],
                // fourth row
                sob_array_y[i - 3 + w * j],
                sob_array_y[i - 2 + w * j],
                sob_array_y[i + 2 + w * j],
                sob_array_y[i + 3 + w * j],
                // fifth row
                sob_array_y[i - 3 + w * (j + 1)],
                sob_array_y[i - 2 + w * (j + 1)],
                sob_array_y[i + 2 + w * (j + 1)],
                sob_array_y[i + 3 + w * (j + 1)],
                // sixth row
                sob_array_y[i - 3 + w * (j + 2)],
                sob_array_y[i - 2 + w * (j + 2)],
                sob_array_y[i - 1 + w * (j + 2)],
                sob_array_y[i + w * (j + 2)],
                sob_array_y[i + 1 + w * (j + 2)],
                sob_array_y[i + 2 + w * (j + 2)],
                sob_array_y[i + 3 + w * (j + 2)],
                // seventh row
                sob_array_y[i - 3 + w * (j + 3)],
                sob_array_y[i - 2 + w * (j + 3)],
                sob_array_y[i - 1 + w * (j + 3)],
                sob_array_y[i + w * (j + 3)],
                sob_array_y[i + 1 + w * (j + 3)],
                sob_array_y[i + 2 + w * (j + 3)],
                sob_array_y[i + 3 + w * (j + 3)],
            ];
            //let thresh = quickselect(cfar_sample, 39);
            let thresh_x = quickselect(cfar_sample_x, (CFAR_QUANTILE*40.0) as usize);
            let thresh_y = quickselect(cfar_sample_y, (CFAR_QUANTILE*40.0) as usize);
            //let thresh_x = *cfar_sample_x.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
            //let thresh_y = *cfar_sample_y.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
            if sob_array_x[i + w * j] > thresh_x && sob_array_x[i + w * j] > threshold_x {
                result_x.push((i as f32, j as f32));
            }
            if sob_array_y[i + w * j] > thresh_y && sob_array_y[i + w * j] > threshold_y {
                result_y.push((i as f32, j as f32));
            }
        }
    }

    (result_x, result_y)
}

fn extract_sobel_pointcloud(img: &DynamicImage) -> Vec<(f32, f32)> {
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

    let threshold = quickselect(
        sob_array.clone(),
        (QUANTILE * (sob_array.len() as f32)) as usize,
    );

    let mut result = vec![];
    let w = img.width() as usize;
    for i in 3..(w - 3) {
        for j in 3..((img.height() - 3) as usize) {
            if sob_array[i + w * j] > threshold {
                result.push((i as f32, j as f32));
            }
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
            (55 + ((113 * (i + 1)) % 200)) as u8,
            (55 + ((127 * (i + 2)) % 200)) as u8,
            (55 + ((131 * (i + 3)) % 200)) as u8,
        ])
    };

    let mut blackboard = RgbImage::new(3 * width, height);

    for x in 0..width {
        for y in 0..height {
            blackboard.put_pixel(x, y, img.get_pixel(x, y).to_rgb());
        }
    }

    for pixel in sobel_pointcloud.iter() {
        blackboard.put_pixel(
            width + (pixel.0 as u32),
            pixel.1 as u32,
            Rgb([255, 255, 255]),
        );
    }

    //*
    for px in 0..width {
        for py in 0..height {
            for (n, seg) in segments.iter().enumerate() {
                if seg.distance((px, py)) < 1.5 {
                    blackboard.put_pixel(2 * width + px, py, get_col(n));
                }
            }
        }
    }
    //*/
    /*
    for (i, seg) in segments.iter().enumerate() {
        let min_x = i64::max(0, i64::min(seg.start.0 as i64, seg.end.0 as i64) - 1);
        let max_x = i64::max(seg.start.0 as i64, seg.end.0 as i64) + 1;
        for px in min_x..max_x {
            let min_y = i64::max(0, i64::min(seg.start.1 as i64, seg.end.1 as i64) - 1);
            let max_y = i64::max(seg.start.1 as i64, seg.end.1 as i64) + 1;
            for py in min_y..max_y {
                if seg.distance((px as u32, py as u32)) < 1.5 {
                    blackboard.put_pixel(px as u32, py as u32, get_col(i));
                }
            }
        }
    }
    */

    /*
    for (i, segment) in segments.iter().enumerate() {
        let pixels = segment.list_all_pixels(width as usize);
        for px in pixels.iter() {
            if (px.0 as u32) < width && (px.1 as u32) < height {
                blackboard.put_pixel(2*width + px.0 as u32, px.1 as u32, get_col(i));
            }
        }
    }
    */

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
    let img = ImageReader::open(IN_FILE_NAME)?.decode()?;
    let (mut sobel_x, sobel_y) = extract_sobel_cfar(&img);
    //println!("{}", edges.len());
    //println!("{:?}", edges);
    let mindim = u32::min(img.height(), img.width());
    let minlen = (mindim as f32)/32.0;
    assert!(mindim > 32);
    let adaptive_depth = (f32::log2(mindim as f32) as u32) - 4;
    let now = Instant::now();
    let quadtree_pts_x = quadtree_cluster(&sobel_x, adaptive_depth);
    let quadtree_pts_y = quadtree_cluster(&sobel_y, adaptive_depth);
    let mut edges = detect_edges(&quadtree_pts_x, (adaptive_depth - 1) as usize, minlen);
    edges.extend(detect_edges(&quadtree_pts_y, (adaptive_depth - 1) as usize, minlen));
    let elapsed = now.elapsed();
    println!(
        "{} edges detected in {} milliseconds",
        edges.len(),
        elapsed.as_millis()
    );

    sobel_x.extend(sobel_y);
    let new_img = draw_segments(&img, &sobel_x, &edges, img.width(), img.height());
    //let new_img = draw_points(&img, &quadtree_pts, img.width(), img.height());

    new_img.save(OUT_FILE_NAME)?;

    Ok(())
}

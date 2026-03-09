use edge_segmentation::segment::*;

use image::ImageReader;
use plotters::prelude::*;

const OUT_FILE_NAME: &str = "test.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = ImageReader::open("funhouse.jpg")?.decode()?.into_rgb8();

    let root = BitMapBackend::new(OUT_FILE_NAME, (img.dimensions().0, img.dimensions().1))
        .into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_cartesian_2d(0..img.dimensions().0, 0..img.dimensions().1)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let plotting_area = chart.plotting_area();

    for px in 0..img.dimensions().0 {
        for py in 0..img.dimensions().1 {
            let col = img.get_pixel(px as u32, py as u32);
            plotting_area.draw_pixel(
                (
                    px,//px as f64 / img.dimensions().0 as f64,
                    img.dimensions().1 - py//1.0 - py as f64 / img.dimensions().1 as f64,
                ),
                &RGBColor(col.0[0], col.0[1], col.0[2]),
            )?;
        }
    }

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Root not present?");
    println!("Result has been saved to {}", OUT_FILE_NAME);

    Ok(())
}

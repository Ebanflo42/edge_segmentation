#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub direction: (f32, f32),
}

impl Segment {
    pub fn new(start: (usize, usize), end: (usize, usize)) -> Self {
        Segment {
            start,
            end,
            direction: (
                (end.0 as f32) - (start.0 as f32),
                (end.1 as f32) - (start.1 as f32),
            ),
        }
    }

    // assumes that end has greater y coordinate than start, so direction.1 is always positive
    pub fn count_in_pixels(&self, img: &[bool], width: usize) -> usize {
        //dbg!(self);
        let mut n_pixels = img[self.start.0 + width*self.start.1] as usize;

        let left = self.direction.0 < 0.0;
        let y_slope = self.direction.1 / self.direction.0;
        let x_slope = self.direction.0 / self.direction.1;

        // these functions only need to work for positive x
        fn better_floor(x: f32) -> f32 {
            if x.fract() <= f32::EPSILON {
                x - 1.0
            } else {
                x.floor()
            }
        }

        fn better_ceil(x: f32) -> f32 {
            if x.fract() <= f32::EPSILON {
                x + 1.0
            } else {
                x.ceil()
            }
        }

        let (mut a, mut b) = (self.start.0 as f32, self.start.1 as f32);
        for _ in 0..2*width {
            (a, b) = if left {
                (f32::max(better_floor(a), a + x_slope),
                 f32::min(better_ceil(b), b - y_slope))
            } else {
                (f32::min(better_ceil(a), a + x_slope),
                 f32::min(better_ceil(b), b + y_slope))
            };

            // hacky fix to FP imprecision
            a = if a.fract() > 0.9999 {
                a.ceil()
            } else {
                a
            };
            b = if b.fract() > 0.9999 {
                b.ceil()
            } else {
                b
            };

            //println!("(a, b) = {} {}", a, b);
            let ix = (a as usize, b as usize);
            //println!("Index: {:?}", ix);
            if img[ix.0 + width * ix.1] {
                //println!("Increment!");
                n_pixels += 1;
            }

            if ix.1 >= self.end.1 {
                break;
            }
        }

        n_pixels
    }
}

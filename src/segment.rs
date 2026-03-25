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
        let mut n_pixels = img[self.start.0 + width * self.start.1] as usize;

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
        for _ in 0..2 * width {
            (a, b) = if left {
                (
                    f32::max(better_floor(a), a + x_slope),
                    f32::min(better_ceil(b), b - y_slope),
                )
            } else {
                (
                    f32::min(better_ceil(a), a + x_slope),
                    f32::min(better_ceil(b), b + y_slope),
                )
            };

            // hacky fix to FP imprecision
            a = if a.fract() > 0.9999 { a.ceil() } else { a };
            b = if b.fract() > 0.9999 { b.ceil() } else { b };

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

    // assumes that end has greater y coordinate than start, so direction.1 is always positive
    pub fn list_in_pixels(&self, img: &[bool], width: usize) -> Vec<(usize, usize)> {
        //dbg!(self);
        let mut pixels = Vec::new();

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
        for _ in 0..2 * width {
            (a, b) = if left {
                (
                    f32::max(better_floor(a), a + x_slope),
                    f32::min(better_ceil(b), b - y_slope),
                )
            } else {
                (
                    f32::min(better_ceil(a), a + x_slope),
                    f32::min(better_ceil(b), b + y_slope),
                )
            };

            // hacky fix to FP imprecision
            a = if a.fract() > 0.9999 { a.ceil() } else { a };
            b = if b.fract() > 0.9999 { b.ceil() } else { b };

            //println!("(a, b) = {} {}", a, b);
            let ix = (a as usize, b as usize);
            //println!("Index: {:?}", ix);
            if img[ix.0 + width * ix.1] {
                //println!("Increment!");
                pixels.push(ix);
            }

            if ix.1 >= self.end.1 {
                break;
            }
        }

        pixels
    }

    // assumes that end has greater y coordinate than start, so direction.1 is always positive
    pub fn list_all_pixels(&self, width: usize) -> Vec<(usize, usize)> {
        //dbg!(self);
        let mut pixels = Vec::new();

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
        for _ in 0..2 * width {
            (a, b) = if left {
                (
                    f32::max(better_floor(a), a + x_slope),
                    f32::min(better_ceil(b), b - y_slope),
                )
            } else {
                (
                    f32::min(better_ceil(a), a + x_slope),
                    f32::min(better_ceil(b), b + y_slope),
                )
            };

            // hacky fix to FP imprecision
            a = if a.fract() > 0.9999 { a.ceil() } else { a };
            b = if b.fract() > 0.9999 { b.ceil() } else { b };

            //println!("(a, b) = {} {}", a, b);
            let ix = (a as usize, b as usize);
            //println!("Index: {:?}", ix);
            pixels.push(ix);

            if ix.1 >= self.end.1 {
                break;
            }
        }

        pixels
    }

    pub fn distance(&self, pixel: (u32, u32)) -> f32 {
        let p = (
            (pixel.0 as f32) - (self.start.0 as f32),
            (pixel.1 as f32) - (self.start.1 as f32),
        );
        // distance start to end
        let d = (self.direction.0 as f32, self.direction.1 as f32);
        let dn = f32::sqrt(d.0 * d.0 + d.1 * d.1);
        // project pixel onto direction vector
        let dp = (p.0 * d.0 + p.1 * d.1) / dn;
        if dp < 0.0 {
            // distance pixel to start
            f32::sqrt(p.0 * p.0 + p.1 * p.1)
        } else if dp < dn {
            // project onto orthogonal vector and take norm
            f32::abs((-d.1 * p.0 + d.0 * p.1) / dn)
        } else {
            let dd = (d.0 - p.0, d.1 - p.1);
            f32::sqrt(dd.0 * dd.0 + dd.1 * dd.1)
        }
    }

    pub fn maybe_extend(&self, other: &Segment) -> Option<Segment> {
        if i32::max(
            i32::abs(other.start.0 as i32 - self.end.0 as i32),
            i32::abs(other.start.1 as i32 - self.end.1 as i32),
        ) < 16
        {
            let d1 = (self.direction.0 as f32, self.direction.1 as f32);
            let d2 = (other.direction.0 as f32, other.direction.1 as f32);
            let dp = (d1.0 * d2.0 + d1.1 * d2.1)
                / f32::sqrt((d1.0 * d1.0 + d1.1 * d1.1) * (d2.0 * d2.0 + d2.1 * d2.1));
            if dp > 0.99 || dp < -0.99 {
                Some(Self::new(self.start, other.end))
            } else {
                None
            }
        } else if i32::max(
            i32::abs(other.end.0 as i32 - self.start.0 as i32),
            i32::abs(other.end.1 as i32 - self.start.1 as i32),
        ) < 16
        {
            let d1 = (self.direction.0 as f32, self.direction.1 as f32);
            let d2 = (other.direction.0 as f32, other.direction.1 as f32);
            let dp = (d1.0 * d2.0 + d1.1 * d2.1)
                / f32::sqrt((d1.0 * d1.0 + d1.1 * d1.1) * (d2.0 * d2.0 + d2.1 * d2.1));
            if dp > 0.99 || dp < -0.99 {
                Some(Self::new(other.start, self.end))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn length(&self) -> f32 {
        f32::sqrt(self.direction.0 * self.direction.0 + self.direction.1 * self.direction.1)
    }
}

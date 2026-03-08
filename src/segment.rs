#[derive(Clone, Copy)]
pub struct Segment {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub direction: (f32, f32),
}

impl Segment {
    pub fn new(start: (usize, usize), end: (usize, usize)) -> Self {
        Segment{ start, end, direction: ((end.0 - start.0) as f32, (end.1 - start.1) as f32) }
    }

    // assumes that end has greater y coordinate than start, so direction.1 is always positive
    pub fn count_in_pixels(&self, img: &[bool], width: usize) -> usize {
        let mut n_pixels = 1;

        let (mut a, mut b) = (self.start.0 as f32, self.start.1 as f32);
        let end_y = self.end.1 as f32;
        while end_y - b > 0.0 {
            (a, b) = if f32::fract(b) > 0.0 {
                let ceilb = f32::ceil(b);
                let step_x = (ceilb - b) * self.direction.0 / self.direction.1;
                (a + step_x, ceilb)
            } else {
                let ceila = f32::ceil(a);
                let step_y = (ceila - a) * self.direction.1 / self.direction.0;
                (ceila, b + step_y)
            };

            if img[a as usize + width * (b as usize)] {
                n_pixels += 1;
            }
        }

        n_pixels
    }
}

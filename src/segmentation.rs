use crate::segment::*;

#[derive(Clone, Copy)]
struct TrackedSegment {
    min_start_x: usize,
    max_start_x: usize,
    best_start_x: usize,
    min_end_x: usize,
    max_end_x: usize,
    best_end_x: usize,
    start_y: usize,
    best_end_y: usize,
    start_x: usize,
    end_x: usize,
    stage_zero: bool,
    score: usize,
}

impl TrackedSegment {
    fn new(min_start_x: usize, max_start_x: usize, row: usize) -> Self {
        TrackedSegment {
            min_start_x,
            max_start_x,
            best_start_x: min_start_x,
            min_end_x: min_start_x,
            max_end_x: max_start_x,
            best_end_x: max_start_x,
            start_y: row,
            best_end_y: row,
            start_x: min_start_x,
            end_x: max_start_x,
            stage_zero: true,
            score: 0,
        }
    }

    fn update(&mut self, candidate: Segment, img: &[bool], width: usize) -> bool {
        let new_score = candidate.count_in_pixels(img, width);
        if new_score > self.score {
            self.score = new_score;
            self.best_end_x = candidate.end.0;
            self.best_end_y = candidate.end.1;
            if candidate.end.0 < self.min_end_x {
                self.min_end_x = candidate.end.0;
            }
            if candidate.end.0 > self.max_end_x {
                self.max_end_x = candidate.end.0;
            }
            true
        } else {
            false
        }
    }

    fn extrapolate_likely_interval(&mut self, row: usize, width: usize) -> (usize, usize) {
        // in this case, the edge was only scanned for one row so far, so we extrapolate a wide interval
        if self.stage_zero {
            self.stage_zero = false;
            (
                if self.max_start_x > 2 * self.min_start_x {
                    0
                } else {
                    2 * self.min_start_x - self.max_start_x
                },
                if 2 * self.max_start_x > width + self.max_start_x {
                    width
                } else {
                    2 * self.max_start_x - self.min_start_x
                },
            )
        } else {
            let dy = (row - self.start_y) as f32;
            let dx_dy1 = ((self.max_start_x - self.min_start_x) as f32) / dy;
            let dx_dy2 = ((self.max_end_x - self.min_start_x) as f32) / dy;
            (
                usize::max(
                    0,
                    self.min_start_x + (f32::floor(f32::min(dx_dy1, dx_dy2)) as usize),
                ),
                usize::min(
                    width,
                    self.max_end_x + (f32::ceil(f32::max(dx_dy1, dx_dy2)) as usize),
                ),
            )
        }
    }
}

pub fn segment_edges(img: &[bool], height: usize, width: usize) -> Vec<Segment> {
    let mut edge_segments = Vec::<TrackedSegment>::with_capacity(128);
    let mut extrapolated_intervals = Vec::<(usize, usize, usize)>::with_capacity(128);
    let mut needs_updating = Vec::<usize>::with_capacity(128);
    let mut updated_this_row = Vec::<usize>::with_capacity(128);

    for j in 0..height {
        updated_this_row.clear();
        let mut new_edge_id = -1i8;
        let mut new_edge_start = 0usize;

        for i in 0..width {
            if img[i + width * j] {
                let plausible_edges: Vec<usize> = extrapolated_intervals
                    .iter()
                    .filter(|(_, i1, i2)| *i1 <= i || i <= *i2)
                    .map(|x| x.0)
                    .collect();

                let mut matched_edge = false;
                for edge_idx in plausible_edges.iter() {
                    let this_segment = edge_segments[*edge_idx];
                    let candidate =
                        Segment::new((this_segment.best_start_x, this_segment.start_y), (i, j));
                    if edge_segments[*edge_idx].update(candidate, &img, width) {
                        matched_edge = true;
                        needs_updating
                            .remove(*needs_updating.iter().find(|x| *x == edge_idx).unwrap());
                        updated_this_row.push(*edge_idx);
                    }
                }

                if !matched_edge && new_edge_id == -1 {
                    new_edge_id = edge_segments.len() as i8;
                    new_edge_start = i;
                }
            } else if new_edge_id > -1 {
                let new_edge = TrackedSegment::new(new_edge_start, i, j);
                updated_this_row.push(edge_segments.len());
                edge_segments.push(new_edge);
            }
        }

        edge_segments = edge_segments
            .iter()
            .enumerate()
            .filter(|(ix, edge)| !needs_updating.iter().any(|e| e == ix) || edge.score > 8)
            .map(|x| *x.1)
            .collect();
    }

    vec![]
}

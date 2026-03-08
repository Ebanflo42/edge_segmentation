use crate::segment::*;

#[derive(Clone, Copy, PartialEq)]
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
        self.stage_zero = false;

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

    fn best_segment(&self) -> (Segment, usize) {
        (
            Segment::new(
                (self.best_start_x, self.start_y),
                (self.best_end_x, self.best_end_y),
            ),
            self.score,
        )
    }
}

pub fn segment_edges(img: &[bool], height: usize, width: usize) -> Vec<(Segment, usize)> {
    let mut edge_segments: [Option<TrackedSegment>; 128] = [None; 128];
    let mut extrapolated_intervals = Vec::with_capacity(128);
    let mut needs_updating = [false; 128];
    let mut updated_this_row = Vec::<usize>::with_capacity(128);
    let mut least_unoccupied_index = 0;

    for j in 0..height {
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
                    let this_segment = edge_segments[*edge_idx].unwrap();
                    let candidate =
                        Segment::new((this_segment.best_start_x, this_segment.start_y), (i, j));
                    if edge_segments[*edge_idx]
                        .unwrap()
                        .update(candidate, &img, width)
                    {
                        matched_edge = true;
                        needs_updating[*edge_idx] = false;
                        updated_this_row.push(*edge_idx);
                    }
                }

                if !matched_edge && new_edge_id == -1 {
                    new_edge_id = least_unoccupied_index as i8;
                    new_edge_start = i;
                }
            } else if new_edge_id > -1 {
                let new_edge = TrackedSegment::new(new_edge_start, i, j);
                updated_this_row.push(new_edge_id as usize);
                edge_segments[new_edge_id as usize] = Some(new_edge);

                while edge_segments[least_unoccupied_index] != None {
                    least_unoccupied_index += 1;
                }
            }
        }

        // remove edges covering less than 8 pixels which were not updated this row
        for id in (0..128).rev() {
            match edge_segments[id] {
                None => continue,
                Some(e) => {
                    if needs_updating[id] && e.score < 8 {
                        edge_segments[id] = None;
                        least_unoccupied_index = id;
                    }
                }
            }
            needs_updating[id] = false;
        }

        // extrapolate likely intervals for the edges which were updated
        if j < height - 1 {
            extrapolated_intervals.clear();
            for id in updated_this_row.drain(..) {
                let interval = edge_segments[id]
                    .unwrap()
                    .extrapolate_likely_interval(j, width);
                extrapolated_intervals.push((id, interval.0, interval.1));
                needs_updating[id] = true;
            }
        }
    }

    edge_segments
        .iter()
        .filter(|e| **e != None)
        .map(|e| e.unwrap().best_segment())
        .collect()
}

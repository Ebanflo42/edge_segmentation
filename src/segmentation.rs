use crate::segment::*;

// data structure representing a line segment with "confidence intervals"
// on its start and endpoints
#[derive(Clone, Copy, PartialEq, Debug)]
struct TrackedSegment {
    initialized: bool,
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
    fn default() -> Self {
        TrackedSegment {
            initialized: false,
            min_start_x: 0,
            max_start_x: 0,
            best_start_x: 0,
            min_end_x: 0,
            max_end_x: 0,
            best_end_x: 0,
            start_y: 0,
            best_end_y: 0,
            start_x: 0,
            end_x: 0,
            stage_zero: false,
            score: 0,
        }
    }

    fn new(min_start_x: usize, max_start_x: usize, row: usize) -> Self {
        //println!("new segment: {} {} {}", min_start_x, max_start_x, row);
        TrackedSegment {
            initialized: true,
            min_start_x,
            max_start_x,
            best_start_x: min_start_x,
            min_end_x: min_start_x,
            max_end_x: max_start_x,
            best_end_x: max_start_x,
            start_y: row,
            best_end_y: row + 1,
            start_x: min_start_x,
            end_x: max_start_x,
            stage_zero: true,
            score: max_start_x - min_start_x,
        }
    }

    fn update(&mut self, candidate: Segment, img: &[bool], width: usize) -> bool {
        self.stage_zero = false;

        let new_score = candidate.count_in_pixels(img, width);
        /*
        println!(
            "candidate, score, old_score: {:?} {} {}",
            candidate, new_score, self.score
        );
        */
        if new_score > self.score {
            //println!("updating!");
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
                if 2 * self.max_start_x > width + self.min_start_x {
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
                f32::max(
                    0.0,
                    (self.min_start_x as f32) + f32::floor(f32::min(dx_dy1, dx_dy2)) - 1.0,
                ) as usize,
                usize::min(
                    width,
                    self.max_start_x + (f32::ceil(f32::max(dx_dy1, dx_dy2)) as usize + 1),
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

pub fn segment_edges(
    img: &[bool],
    height: usize,
    width: usize,
    min_pixels_per_edge: usize,
) -> Vec<(Segment, usize)> {
    let mut edge_segments = [TrackedSegment::default(); 128];
    let mut extrapolated_intervals = Vec::with_capacity(128);
    let mut needs_updating = [false; 128];
    let mut updated_this_row = [false; 128];
    let mut least_unoccupied_index = 0;

    for j in 0..(height - 1) {
        //println!("row: {}", j);
        let mut new_edge_id = -1i8;
        let mut new_edge_start = 0usize;
        for id in 0..128 {
            updated_this_row[id] = false;
        }
        //println!("{:?}", extrapolated_intervals);
        /*
        println!(
            "{:?}",
            edge_segments
                .clone()
                .iter()
                .filter(|x| x.initialized)
                .map(|x| x.best_segment())
                .collect::<Vec<(Segment, usize)>>()
        );
        */

        for i in 0..(width - 1) {
            if img[i + width * j] {
                let plausible_edges: Vec<(usize, usize, usize)> = extrapolated_intervals
                    .iter()
                    .filter(|(_, i1, i2)| *i1 <= i && i <= *i2)
                    .map(|x| *x)
                    .collect();

                if plausible_edges.len() > 0 {
                    //println!("candidate pixel: {} {}", i, j);
                    for &interval in plausible_edges.iter() {
                        //println!("{} {}", interval.1, interval.2);
                        let this_segment = edge_segments[interval.0];
                        if !this_segment.stage_zero {
                            let candidate = Segment::new(
                                (this_segment.best_start_x, this_segment.start_y),
                                (i, j),
                            );
                            if edge_segments[interval.1].update(candidate, &img, width) {
                                //matched_edge = true;
                                needs_updating[interval.0] = false;
                                updated_this_row[interval.0] = true;
                            }
                        } else {
                            let candidate1 = Segment::new(
                                (this_segment.min_start_x, this_segment.start_y),
                                (i, j),
                            );
                            let candidate2 = Segment::new(
                                (this_segment.max_start_x, this_segment.start_y),
                                (i, j),
                            );
                            if edge_segments[interval.0].update(candidate1, &img, width)
                                || edge_segments[interval.0].update(candidate2, &img, width)
                            {
                                //matched_edge = true;
                                needs_updating[interval.0] = false;
                                updated_this_row[interval.0] = true;
                            }
                        }
                    }
                } else if new_edge_id == -1 {
                    new_edge_id = least_unoccupied_index as i8;
                    new_edge_start = i;
                }
            } else if new_edge_id > -1 {
                let new_edge = TrackedSegment::new(new_edge_start, i, j);
                updated_this_row[new_edge_id as usize] = true;
                edge_segments[new_edge_id as usize] = new_edge;

                while edge_segments[least_unoccupied_index].initialized {
                    least_unoccupied_index += 1;
                }

                new_edge_id = -1;
            }
        }

        if new_edge_id > -1 {
            let new_edge = TrackedSegment::new(new_edge_start, width - 1, j);
            updated_this_row[new_edge_id as usize] = true;
            edge_segments[new_edge_id as usize] = new_edge;

            while edge_segments[least_unoccupied_index].initialized {
                least_unoccupied_index += 1;
            }
        }

        // remove edges covering less than 8 pixels which were not updated this row
        for id in (0..128).rev() {
            if edge_segments[id].initialized {
                if needs_updating[id] && edge_segments[id].score < min_pixels_per_edge {
                    edge_segments[id].initialized = false;
                    least_unoccupied_index = id;
                }
            }
            needs_updating[id] = false;
        }

        // extrapolate likely intervals for the edges which were updated
        if j < height - 1 {
            extrapolated_intervals.clear();
            for id in 0..128 {
                if updated_this_row[id] {
                    let interval = edge_segments[id].extrapolate_likely_interval(j, width);
                    extrapolated_intervals.push((id, interval.0, interval.1));
                    needs_updating[id] = true;
                }
            }
        }
    }

    edge_segments
        .iter()
        .filter(|e| e.initialized)
        .map(|e| e.best_segment())
        .collect()
}

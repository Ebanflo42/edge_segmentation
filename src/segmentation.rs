use crate::segment::*;

#[derive(Clone)]
struct TrackedSegment {
    segment: Segment,
    score: usize,
}

pub fn segment_edges(img: &[bool], height: usize, width: usize) -> Vec<Segment> {
    let mut cluster_ids = vec![-1i8; height * width];
    let mut cluster_pixels = vec![Vec::<(usize, usize)>::new(); 16];
    let mut cluster_segments = vec![Vec::<TrackedSegment>::with_capacity(8); 16];
    let mut cluster_updated_this_row = [false; 16];
    let mut least_unoccupied_cluster = 0usize;

    for j in 1..(height - 1) {
        for i in 0..16 {
            cluster_updated_this_row[i] = false;
        }

        for i in 1..(width - 1) {
            if img[i + width * j] {
                let mut found_clusters_at = Vec::<(i8, usize, usize)>::with_capacity(8);

                for di in -1..2 {
                    for dj in -1..2 {
                        if di != 0 || dj != 0 {
                            let neighbor_x = ((i as i64) + di) as usize;
                            let neighbor_y = ((j as i64) + dj) as usize;
                            let cluster_id = cluster_ids[neighbor_x + width * neighbor_y];
                            if cluster_id > -1 {
                                found_clusters_at.push((cluster_id, neighbor_x, neighbor_y));
                            }
                        }
                    }
                }

                if found_clusters_at.len() == 0 {
                    // need a case for cluster ID being greater than max allowable!
                    cluster_ids[i + width * j] = least_unoccupied_cluster as i8;
                    cluster_pixels[least_unoccupied_cluster].push((i, j));
                    least_unoccupied_cluster += 1;
                    // still need another case for cluster merging!!!
                } else {
                    let (cluster_id, x, y) = found_clusters_at[0];
                    cluster_ids[i + width * j] = cluster_id;
                    cluster_pixels[cluster_id as usize].push((i, j));
                    cluster_updated_this_row[cluster_id as usize] = true;

                    if cluster_segments[cluster_id as usize].len() == 0 {
                        let segment = Segment::new((x, y), (i, j));
                        cluster_segments[cluster_id as usize].push(TrackedSegment {
                            segment,
                            score: 2,
                        })
                    } else {
                        let mut min_start_dist = usize::max(height, width) as i64;
                        let mut best_start_ix = 0usize;
                        let mut min_end_dist = usize::max(height, width) as i64;
                        let mut best_end_ix = 0usize;

                        for (ix, seg) in cluster_segments[cluster_id as usize].iter().enumerate() {
                            let next_start_dist = i64::max(
                                i64::abs((seg.segment.start.0 as i64) - (i as i64)),
                                i64::abs(seg.segment.start.1 as i64) - (j as i64),
                            );
                            if next_start_dist < min_start_dist {
                                min_start_dist = next_start_dist;
                                best_start_ix = ix;
                            }

                            let next_end_dist = i64::max(
                                i64::abs((seg.segment.end.0 as i64) - (i as i64)),
                                i64::abs(seg.segment.end.1 as i64) - (j as i64),
                            );
                            if next_end_dist < min_end_dist {
                                min_end_dist = next_end_dist;
                                best_end_ix = ix;
                            }
                        }

                        let best_end_segment = &cluster_segments[cluster_id as usize][best_end_ix];
                        let candidate = Segment::new(best_end_segment.segment.start, (i, j));
                        let new_score = candidate.count_in_pixels(&cluster_ids, cluster_id, width);
                        if new_score > best_end_segment.score {
                            cluster_segments[cluster_id as usize][best_end_ix] = TrackedSegment {
                                segment: candidate,
                                score: new_score,
                            }
                        } else {
                            let best_start_segment =
                                &cluster_segments[cluster_id as usize][best_start_ix];
                            let new_segment =
                                Segment::new(best_start_segment.segment.start, (i, j));
                            let score =
                                new_segment.count_in_pixels(&cluster_ids, cluster_id, width);
                            cluster_segments[cluster_id as usize].push(TrackedSegment {
                                segment: new_segment,
                                score,
                            })
                        }
                    }
                }
            }
        }

        // erase clusters with less than 16 pixels that havent been updated
        for cluster_id in (0..16).rev() {
            if !cluster_updated_this_row[cluster_id]
                && cluster_pixels[cluster_id].len() > 0
                && cluster_pixels.len() < 16
            {
                for p in cluster_pixels[cluster_id].iter() {
                    cluster_ids[p.0 + width * p.1] = -1;
                }
                cluster_pixels[cluster_id].clear();
                cluster_segments[cluster_id].clear();
                least_unoccupied_cluster = cluster_id;
            }
        }
    }

    let n_segments = cluster_segments.iter().map(|segs| segs.len()).sum();
    let mut result = Vec::<Segment>::with_capacity(n_segments);
    for segs in cluster_segments.into_iter() {
        for seg in segs.iter() {
            if seg.score > 4 {
                result.push(seg.segment);
            }
        }
    }

    result
}

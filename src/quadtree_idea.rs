#![feature(explicit_tail_calls)]
use crate::segment::Segment;
use std::collections::BTreeSet;
//#[macro_use] extern crate tramp;
//use tramp::{tramp, Rec, rec_call, rec_ret};

//*
fn median_lte5(xs: &Vec<f32>) -> f32 {
    if xs.len() == 1 {
        xs[0]
    } else if xs.len() == 2 {
        0.5 * (xs[0] + xs[1])
    } else if xs.len() == 3 {
        xs.clone().sort_by(|a, b| a.total_cmp(b));
        xs[1]
    } else if xs.len() == 4 {
        xs.clone().sort_by(|a, b| a.total_cmp(b));
        0.5 * (xs[1] + xs[2])
    } else if xs.len() == 5 {
        xs.clone().sort_by(|a, b| a.total_cmp(b));
        xs[2]
    } else {
        panic!("Unintended usage of `median_lte5`!");
    }
}

pub fn quickselect(mut candidate_elems: Vec<f32>, mut i: usize) -> f32 {
    while candidate_elems.len() > 5 {
        //println!("{}", candidate_elems.len());
        let mut candidate_pivots = candidate_elems.clone();
        while candidate_pivots.len() > 5 {
            candidate_pivots = candidate_pivots
                .chunks(5)
                .filter(|xs| xs.len() == 5)
                .map(|xs| {
                    let mut ys = xs.to_vec();
                    ys.sort_by(|a, b| a.total_cmp(b));
                    ys[2]
                })
                .collect();
        }
        let pivot = median_lte5(&candidate_pivots);
        //println!("{:?}, {}", candidate_elems, pivot);

        let mut lessthan = Vec::with_capacity(candidate_elems.len());
        let mut eq_count = 0usize;
        let mut greaterthan = Vec::with_capacity(candidate_elems.len());

        for x in candidate_elems.iter() {
            if *x < pivot {
                lessthan.push(*x);
            } else if *x == pivot {
                eq_count += 1;
            } else {
                greaterthan.push(*x);
            }
        }
        if i < lessthan.len() {
            //println!("LESS THAN {} {}", i, lessthan.len());
            candidate_elems = lessthan;
        } else if i < eq_count + lessthan.len() {
            //println!("EQUALS {} {}", i, pivot);
            return pivot;
        } else {
            //println!("GREATER THAN {} {}", i, greaterthan.len());
            candidate_elems = greaterthan;
            i -= eq_count + lessthan.len();
        }
        //panic!("");
    }
    candidate_elems[i]
}

fn linear_time_median(xs: Vec<f32>) -> f32 {
    let l = xs.len();
    if xs.len() % 2 == 1 {
        quickselect(xs, l / 2)
    } else {
        0.5 * (quickselect(xs.clone(), l / 2) + quickselect(xs, l / 2 - 1))
    }
}

fn medioid(points: &Vec<(f32, f32)>) -> (f32, f32) {
    (
        linear_time_median(points.iter().map(|x| x.0).collect::<Vec<f32>>()),
        linear_time_median(points.iter().map(|x| x.1).collect::<Vec<f32>>()),
    )
}

//*/
fn centroid(points: &Vec<(f32, f32)>) -> (f32, f32) {
    (
        points.iter().map(|x| x.0).sum::<f32>() / (points.len() as f32),
        points.iter().map(|x| x.1).sum::<f32>() / (points.len() as f32),
    )
}

fn covariance(points: &Vec<(f32, f32)>, c: (f32, f32)) -> (f32, f32, f32) {
    let degs_of_freedom = (points.len() - 1) as f32;
    let px = points.iter().map(|p| p.0 - c.0).collect::<Vec<f32>>();
    let py = points.iter().map(|p| p.1 - c.1).collect::<Vec<f32>>();
    (
        px.iter().zip(px.iter()).map(|x| x.0 * x.1).sum::<f32>() / degs_of_freedom,
        px.iter().zip(py.iter()).map(|x| x.0 * x.1).sum::<f32>() / degs_of_freedom,
        py.iter().zip(py.iter()).map(|x| x.0 * x.1).sum::<f32>() / degs_of_freedom,
    )
}

fn subdivide_points(points: &Vec<(f32, f32)>) -> Vec<Vec<(f32, f32)>> {
    let c = centroid(points);
    let mut result = vec![Vec::with_capacity(points.len() / 4); 4];
    for p in points.iter() {
        if p.0 < c.0 && p.1 < c.1 {
            result[0].push(*p);
        } else if p.0 >= c.0 && p.1 < c.1 {
            result[1].push(*p);
        } else if p.0 < c.0 && p.1 >= c.1 {
            result[2].push(*p);
        } else {
            result[3].push(*p);
        }
    }
    result
}

pub fn quadtree_cluster(pointcloud: &Vec<(f32, f32)>, depth: u32) -> Vec<Vec<(f32, f32)>> {
    let mut result = subdivide_points(&pointcloud);
    for _ in 1..depth {
        let mut helper = Vec::with_capacity(4 * result.len());
        for ps in result.drain(..) {
            helper.append(&mut subdivide_points(&ps));
        }
        result = helper;
    }
    result
}

fn solve_kernel(x11: f32, x12: f32) -> (f32, f32) {
    // we assume that the matrix is symmetric and has a 1-dimensional kernel
    // so actually we have only one case for the vector defining the kernel
    let v = (-x12, x11);
    let n = f32::sqrt(v.0 * v.0 + v.1 * v.1);
    (v.0 / n, v.1 / n)
}

fn extract_start_and_end(
    cluster: &Vec<(f32, f32)>,
    small_eig: (f32, f32),
    big_eig: (f32, f32),
) -> ((f32, f32), (f32, f32)) {
    let small_coord = cluster
        .iter()
        .map(|x| {
            let r = x.0 * small_eig.0 + x.1 * small_eig.1;
            r * r
        })
        .collect::<Vec<f32>>();
    let big_coord = cluster
        .iter()
        .map(|x| {
            let r = x.0 * big_eig.0 + x.1 * big_eig.1;
            r * r
        })
        .collect::<Vec<f32>>();
    let start_ix = big_coord
        .iter()
        .zip(small_coord.iter())
        .map(|x| x.0 - x.1)
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap()
        .0;
    let end_ix = big_coord
        .iter()
        .zip(small_coord.iter())
        .map(|x| -x.0 - x.1)
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap()
        .0;
    (cluster[start_ix], cluster[end_ix])
}

fn merge_segments(
    mut quadtree_segments: Vec<Vec<Segment>>,
    depth: usize,
    min_length: f32,
) -> Vec<Segment> {
    fn merge_chunk(chunk: &[Vec<Segment>]) -> Vec<Segment> {
        let n = chunk[0].len() + chunk[1].len() + chunk[2].len() + chunk[3].len();
        // remember which segments we merge
        let mut accounted = [
            vec![false; chunk[0].len()],
            vec![false; chunk[1].len()],
            vec![false; chunk[2].len()],
            vec![false; chunk[3].len()],
        ];
        let mut result = Vec::with_capacity(n);

        for (i, seg0) in chunk[0].iter().enumerate() {
            for (j, seg1) in chunk[1].iter().enumerate() {
                match seg0.maybe_extend(seg1) {
                    None => continue,
                    Some(s) => {
                        result.push(s);
                        accounted[0][i] = true;
                        accounted[1][j] = true;
                        break;
                    }
                }
            }
            if accounted[0][i] {
                continue;
            }
            for (j, seg2) in chunk[2].iter().enumerate() {
                match seg0.maybe_extend(seg2) {
                    None => continue,
                    Some(s) => {
                        result.push(s);
                        accounted[0][i] = true;
                        accounted[2][j] = true;
                        break;
                    }
                }
            }
            if accounted[0][i] {
                continue;
            }
            for (j, seg3) in chunk[3].iter().enumerate() {
                match seg0.maybe_extend(seg3) {
                    None => continue,
                    Some(s) => {
                        result.push(s);
                        accounted[0][i] = true;
                        accounted[3][j] = true;
                        break;
                    }
                }
            }
            if accounted[0][i] {
                continue;
            }
            result.push(*seg0);
        }

        for (i, seg1) in chunk[1].iter().enumerate() {
            if !accounted[1][i] {
                for (j, seg2) in chunk[2].iter().enumerate() {
                    if !accounted[2][j] {
                        match seg1.maybe_extend(seg2) {
                            None => continue,
                            Some(s) => {
                                result.push(s);
                                accounted[1][i] = true;
                                accounted[2][j] = true;
                                break;
                            }
                        }
                    }
                }
                if accounted[1][i] {
                    continue;
                }
                for (j, seg3) in chunk[3].iter().enumerate() {
                    if !accounted[3][j] {
                        match seg1.maybe_extend(seg3) {
                            None => continue,
                            Some(s) => {
                                result.push(s);
                                accounted[1][i] = true;
                                accounted[3][j] = true;
                                break;
                            }
                        }
                    }
                }
                if accounted[1][i] {
                    continue;
                }
                result.push(*seg1);
            }
        }

        for (i, seg2) in chunk[2].iter().enumerate() {
            if !accounted[2][i] {
                for (j, seg3) in chunk[3].iter().enumerate() {
                    if !accounted[3][j] {
                        match seg2.maybe_extend(seg3) {
                            None => continue,
                            Some(s) => {
                                result.push(s);
                                accounted[2][i] = true;
                                accounted[3][j] = true;
                                break;
                            }
                        }
                    }
                }
                if accounted[2][i] {
                    continue;
                }
                result.push(*seg2);
            }
        }

        for (i, seg3) in chunk[3].iter().enumerate() {
            if !accounted[3][i] {
                result.push(*seg3);
            }
        }

        result
    }

    for _ in 0..depth {
        quadtree_segments = quadtree_segments.chunks(4).map(merge_chunk).collect();
    }

    quadtree_segments
        .into_iter()
        .flatten()
        .filter(|s| s.length() > min_length)
        .collect()
}

pub fn detect_edges(
    clusters: &Vec<Vec<(f32, f32)>>,
    merge_depth: usize,
    min_length: f32,
) -> Vec<Segment> {
    let mut result = vec![Vec::with_capacity(2); clusters.len()];
    for (i, cluster) in clusters.iter().enumerate() {
        if cluster.len() > 2 {
            let mu = centroid(cluster);
            let cov = covariance(&cluster, mu);
            let b = -(cov.0 + cov.2);
            let c = cov.0 * cov.2 - cov.1 * cov.1;
            let discriminant = f32::sqrt(b * b - 4.0 * c);
            let small_eig = 0.5 * (-b - discriminant);
            let big_eig = 0.5 * (-b + discriminant);
            //assert!(small_eig >= 0.0);
            if small_eig < 2.0 && big_eig > 4.0 * small_eig {
                let big_eigvec = solve_kernel(cov.0 - big_eig, cov.1);
                let small_eigvec = solve_kernel(cov.0 - small_eig, cov.1);
                let (mut start, mut end) = extract_start_and_end(cluster, small_eigvec, big_eigvec);
                if f32::abs(start.0 - end.0) < 2.0 && f32::abs(start.1 - end.1) < 2.0 {
                    continue;
                }
                (start, end) = if start.1 < end.1 {
                    (start, end)
                } else {
                    (end, start)
                };
                result[i].push(Segment::new(
                    (start.0 as usize, start.1 as usize),
                    (end.0 as usize, end.1 as usize),
                ));
            }
        } else {
            continue;
        }
    }
    if merge_depth > 0 {
        merge_segments(result, merge_depth, min_length)
    } else {
        result.concat()
    }
}

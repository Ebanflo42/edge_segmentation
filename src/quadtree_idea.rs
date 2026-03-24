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

fn unify_quadtree_segments(preliminary: Vec<Option<Segment>>) -> Vec<Segment> {
    let mut result = Vec::new();
    // fix this bs
    let mut depth = 0;
    while usize::pow(4, depth) != preliminary.len() {
        depth += 1;
    }
    for i in 0..preliminary.len() {
        match preliminary[i] {
            None => continue,
            Some(s) => {
                let dirnorm =
                    f32::sqrt(s.direction.0 * s.direction.0 + s.direction.1 * s.direction.1);
                let dir = (s.direction.0 / dirnorm, s.direction.1 / dirnorm);
                let this_quad = i % 4;
                let opposite_quad = (this_quad + 2)%4;
                let mut parent = i / 4;
                let mut search_depth = 1;
                while search_depth < depth && parent != opposite_quad {
                    parent /= 4;
                    search_depth += 1;
                }
                // the case where we have a cell all the way at the corner of the image
                if search_depth == depth {
                    parent = i / 4;
                    search_depth = 1;
                }
                let n_iters = usize::pow(4, search_depth);
                for j in n_iters * parent..n_iters * (parent + 1) {
                    match preliminary[j] {
                        None => continue,
                        Some(s1) => {
                            if u32::max(
                                s.start.0 as u32 - s1.start.0 as u32,
                                s1.start.1 as u32 - s1.start.1 as u32,
                            ) < 3
                            {
                                let dirnorm1 = f32::sqrt(
                                    s1.direction.0 * s1.direction.0
                                        + s1.direction.1 * s1.direction.1,
                                );
                                let dir1 = (s1.direction.0 / dirnorm1, s1.direction.1 / dirnorm1);
                                // about 5 degrees
                                if dir.0*dir1.0 + dir.1*dir1.1 < 1e-3 {

                                }
                            }
                        }
                    }
                }
            }
        }
    }
    result.into_iter().collect()
}

pub fn detect_edges(clusters: &Vec<Vec<(f32, f32)>>) -> Vec<Segment> {
    let mut result = Vec::with_capacity(clusters.len());
    for cluster in clusters.iter() {
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
                    result.push(None);
                }
                (start, end) = if start.1 < end.1 {
                    (start, end)
                } else {
                    (end, start)
                };
                result.push(Some(Segment::new(
                    (start.0 as usize, start.1 as usize),
                    (end.0 as usize, end.1 as usize),
                )));
            }
        } else {
            result.push(None)
        }
    }
    unify_quadtree_segments(result)
}

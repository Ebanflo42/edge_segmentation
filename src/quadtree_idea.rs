#![feature(explicit_tail_calls)]
use crate::segment::Segment;
//#[macro_use] extern crate tramp;
//use tramp::{tramp, Rec, rec_call, rec_ret};

//*
fn pick_pivot(xs: &Vec<f32>) -> f32 {
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
        let chunk_medians = xs
            .chunks(5)
            .filter(|ys| ys.len() == 5)
            .map(|ys| {
                let mut ysc = ys.to_vec();
                ysc.sort_by(|a, b| a.total_cmp(b));
                ysc[2]
            })
            .collect::<Vec<f32>>();
        linear_time_median(&chunk_medians)
    }
}

fn quickselect(xs: &Vec<f32>, ix: usize) -> f32 {
    let pivot = pick_pivot(xs);
    let mut lessthan = Vec::with_capacity(xs.len());
    let mut greaterthan = Vec::with_capacity(xs.len());
    for x in xs.iter() {
        if *x < pivot {
            lessthan.push(*x);
        } else {
            greaterthan.push(*x);
        }
    }
    if ix < lessthan.len() {
        quickselect(&lessthan, ix)
    } else if ix == lessthan.len() {
        pivot
    } else {
        quickselect(&greaterthan, ix - lessthan.len())
    }
}

fn linear_time_median(xs: &Vec<f32>) -> f32 {
    if xs.len()%2 == 1 {
        quickselect(xs, xs.len()/2)
    } else {
        0.5*(quickselect(xs,xs.len()/2) + quickselect(xs, xs.len()/2 - 1))
    }
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

pub fn detect_edges(clusters: &Vec<Vec<(f32, f32)>>) -> Vec<Segment> {
    let mut result = Vec::with_capacity(clusters.len());
    for cluster in clusters.iter() {
        if cluster.len() > 16 {
            let mu = centroid(cluster);
            let cov = covariance(&cluster, mu);
            let b = -(cov.0 + cov.2);
            let c = cov.0 * cov.2 - cov.1 * cov.1;
            let discriminant = f32::sqrt(b * b - 4.0 * c);
            let small_eig = 0.5 * (-b - discriminant);
            let big_eig = 0.5 * (-b + discriminant);
            //assert!(small_eig >= 0.0);
            if big_eig > 4.0 && small_eig < 2.0 && big_eig > 4.0 * small_eig {
                let big_eigvec = solve_kernel(cov.0 - big_eig, cov.1);
                let small_eigvec = solve_kernel(cov.0 - small_eig, cov.1);
                let (mut start, mut end) = extract_start_and_end(cluster, small_eigvec, big_eigvec);
                (start, end) = if start.1 < end.1 {
                    (start, end)
                } else {
                    (end, start)
                };
                result.push(Segment::new(
                    (start.0 as usize, start.1 as usize),
                    (end.0 as usize, end.1 as usize),
                ));
            }
        }
    }
    result
}

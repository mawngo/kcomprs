// DistanceFunc represents a function for measuring distance between n-dimensional vectors.
pub type DistanceFunc = fn(&[f64; 4], &[f64; 4]) -> f64;

pub fn euclidean_distance(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let mut s = 0.0;
    let mut t: f64;
    for i in 0..a.len() {
        t = a[i] - b[i];
        s += t * t;
    }
    s.sqrt()
}

pub fn euclidean_distance_squared(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let mut s = 0.0;
    let mut t: f64;
    for i in 0..a.len() {
        t = a[i] - b[i];
        s += t * t;
    }
    s
}

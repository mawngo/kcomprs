use crate::kmeans::cluster::DistanceFunc;
use rand::Rng;

pub type Dataset = Vec<[f64; 4]>;

pub struct Trainer {
    pub k: usize,
    pub distance_fn: DistanceFunc,
    pub max_iterations: usize,
    pub delta: f64,
}

impl Trainer {
    pub fn fit(&self, data: Dataset) -> Model {
        let mut model = Model {
            distance_fn: self.distance_fn,
            k: self.k,
            mapping: vec![0; data.len()],
            centroids: vec![[0f64; 4]; self.k],
            iter: 0,
        };
        model.initialize_mean(&data);

        let change_threshold = ((data.len() as f64) * self.delta) as usize;
        let mut cb = vec![0i32; self.k];
        let mut cn = vec![[0f64; 4]; self.k];

        let mut iter = 0;
        loop {
            if iter >= self.max_iterations {
                break;
            }
            iter += 1;

            let mut changes = 0;
            for i in 0..data.len() {
                let mut m = (self.distance_fn)(&model.centroids[0], &data[i]);
                let mut n = 0;
                for j in 1..self.k {
                    let d = (self.distance_fn)(&model.centroids[j], &data[i]);
                    if d < m {
                        m = d;
                        n = j;
                    }
                }

                if model.mapping[i] != n {
                    changes += 1;
                }
                model.mapping[i] = n;
                cb[n] += 1;

                cn[n][0] += &data[i][0];
                cn[n][1] += &data[i][1];
                cn[n][2] += &data[i][2];
                cn[n][3] += &data[i][3];
            }

            for i in 0..self.k {
                let scale = 1.0 / (cb[i] as f64);
                cb[i] = 0;

                cn[i][0] *= scale;
                cn[i][1] *= scale;
                cn[i][2] *= scale;
                cn[i][3] *= scale;

                for j in 0..4 {
                    model.centroids[i][j] = cn[i][j];
                    cn[i][j] = 0.0
                }
            }

            if changes < change_threshold {
                break;
            }
        }

        model.iter = iter;
        model
    }
}

pub struct Model {
    distance_fn: DistanceFunc,
    k: usize,
    pub centroids: Dataset,
    pub mapping: Vec<usize>,
    pub iter: usize,
}

impl Model {
    fn initialize_mean(&mut self, data: &Dataset) {
        self.centroids[0] = data[rand::rng().random_range(0..data.len())];
        let mut d = vec![0f64; data.len()];
        for i in 1..self.k {
            let mut s = 0f64;
            for j in 0..data.len() {
                let mut l = (self.distance_fn)(&self.centroids[0], &data[j]);
                for g in 1..i {
                    let f = (self.distance_fn)(&self.centroids[g], &data[j]);
                    if f < l {
                        l = f
                    }
                }

                d[j] = l * l;
                s += d[j];
            }

            let t = rand::rng().random_range(0.0..1.0) * s;
            let mut k = 0;
            let mut s = d[0];
            loop {
                if s >= t {
                    break;
                }
                k += 1;
                s += d[k];
            }
            self.centroids[i] = data[k]
        }
    }
}

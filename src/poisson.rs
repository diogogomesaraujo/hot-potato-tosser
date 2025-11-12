use rand::{rngs::SmallRng, Rng, SeedableRng};

pub struct Poisson<R: Rng + ?Sized> {
    pub rng: Box<R>,
    pub rate: f64,
}

impl Poisson<SmallRng> {
    pub fn new(rate: f64, seed: &[u8; 32]) -> Self {
        Self {
            rng: Box::new(SmallRng::from_seed(*seed)),
            rate,
        }
    }
    pub fn time_for_next_event(&mut self) -> f64 {
        -(1.0f64 - self.rng.random::<f64>()).log2() / self.rate
    }
}

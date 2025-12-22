use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::Arc,
};

use rand::seq::IndexedRandom;

#[derive(Clone)]
pub struct Splashes(Arc<Vec<String>>);

impl Splashes {
    pub fn new(path: &str) -> Self {
        let splash_path = PathBuf::from(path);
        let file = File::open(splash_path).expect("failed to open splash file");
        let lines = BufReader::new(file).lines();
        Self(Arc::new(lines.map_while(Result::ok).collect()))
    }

    pub fn choose(&self) -> &String {
        self.0.choose(&mut rand::rng()).unwrap()
    }
}

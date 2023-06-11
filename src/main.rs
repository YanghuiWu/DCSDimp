use std::collections::HashMap;
use std::io;
use std::str::FromStr;

use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;

/// Struct for weighted random sampling
struct Generator {
    distribution: WeightedIndex<f64>,
    source: Vec<u64>,
    largest: usize,
}

impl Generator {
    /// Create a new sampler from an iterator of (item, weight) tuples
    fn new<T: Iterator<Item = (u64, f64)>>(t: T) -> Generator {
        let vector: Vec<(u64, f64)> = t.into_iter().collect(); //Guarantees our index ordering.
        let distribution = WeightedIndex::new(vector.iter().map(|(_, weight)| *weight)).unwrap();
        let source: Vec<u64> = vector.into_iter().map(|(item, _)| item).collect();
        let largest = source.iter().max().unwrap().clone() as usize;

        Generator {
            distribution,
            source,
            largest,
        }
    }

    /// Sample an item using the weighted distribution
    fn generate(&self) -> u64 {
        let mut random = rand::thread_rng();
        let index = self.distribution.sample(&mut random);
        self.source[index]
    }
}

/// Struct for caching simulator
struct Simulator {
    size: usize,
    // Track number of expirations for each cache entry
    tracker: HashMap<u64, usize>,
    step: u64,
}

impl Simulator {
    fn init() -> Simulator {
        Simulator {
            size: 0,
            tracker: HashMap::new(),
            step: 0,
        }
    }

    /// Add a new cache tenancy to the simulator
    fn add_tenancy(&mut self, tenancy: u64) {
        self.update();
        self.size += 1;
        let target = tenancy + self.step;
        let expirations_at_step = self.tracker.get(&target).copied().unwrap_or(0);
        self.tracker.insert(target, expirations_at_step + 1);
    }

    fn update(&mut self) {
        self.step += 1;
        let expired = self.tracker.remove(&self.step).unwrap_or(0);
        self.size -= expired;
    }
}

/// Perform the caching simulation
fn caching(ten_dist: Generator, _cache_size: u64, samples_to_issue: usize) -> Vec<u64> {
    let mut cache = Simulator::init();
    let mut dcsd_observed = vec![0; ten_dist.largest + 2];
    let tenancy = ten_dist.generate();
    cache.add_tenancy(tenancy);

    for _cycles in 0..samples_to_issue {
        let tenancy = ten_dist.generate();
        cache.add_tenancy(tenancy);
        dcsd_observed[cache.size] += 1;
    }
    return dcsd_observed.clone();
}

fn input_to_hashmap() -> Generator {
    let mut rdr = csv::Reader::from_reader(io::stdin());
    let mut tenancy_dist: HashMap<u64, f64> = HashMap::new();
    // let mut largest = 0;
    for result in rdr.records() {
        let record = result.unwrap();

        let tenancy = record.get(0).unwrap().parse().unwrap();
        let prob = f64::from_str(&record.get(1).unwrap()).unwrap();

        tenancy_dist.insert(tenancy, prob);
    }
    return Generator::new(tenancy_dist.into_iter());
}

fn write(output: Vec<u64>, samples: usize) {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(&["DCS", "probability"])
        .expect("cannot write");
    output.iter().enumerate().for_each(|(index, value)| {
        let percentage = (*value as f64) / samples as f64 * 100.0;
        let formatted_percentage = format!("{:.4}%", percentage);
        wtr.write_record(&[index.to_string(), formatted_percentage])
            .unwrap();
    });

    wtr.flush().unwrap();
    println!("Samples #: {}", samples);
}

fn main() {
    // let test = input_to_hashmap();
    let tenancy_dist = input_to_hashmap();
    let samples = tenancy_dist.largest * 200000;
    let output = caching(tenancy_dist, 100, samples);
    write(output, samples);
}

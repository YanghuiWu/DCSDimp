use std::collections::HashMap;
use std::io;
use std::str::FromStr;

use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;

/// Struct for weighted random sampling
struct Generator {
    distribution: WeightedIndex<f64>,
    source: Vec<u64>,
}


impl Generator {
    /// Create a new sampler from an iterator of (item, weight) tuples
    fn new<T: Iterator<Item=(u64, f64)>>(t: T) -> Generator {
        let vector: Vec<(u64, f64)> = t.into_iter().collect(); //Guarantees our index ordering.
        let distribution = WeightedIndex::new(vector.iter().map(|(_, weight)| *weight)).unwrap();
        let source = vector.into_iter().map(|(item, _)| item).collect();

        Generator {
            distribution,
            source,
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
    size: u64,
    // Track number of expirations for each cache entry
    tracker: HashMap<u64, u64>,
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
        self.size -= self.tracker.remove(&self.step).unwrap_or(0);
    }
}

/// Perform the caching simulation
fn caching(ten_dist: Generator, _cache_size: u64, _delta: f64, samples_to_issue: usize) -> Vec<u64> {
    let mut cache = Simulator::init();
    // let samples_to_issue: u64 = length as u64;
    // let mut prev_output: Vec<u64> = vec![0; samples_to_issue + 1];
    let mut dcsd_observed = vec![0; samples_to_issue + 1];
    let mut cycles = 0;
    let tenancy = ten_dist.generate();
    cache.add_tenancy(tenancy);

    loop {
        // println!("cycle: {}", cycles);
        if cycles > 100000 {//this is the main loop, larger numbers of loop gives higher precisions
            return dcsd_observed.clone();
        }
        for _ in 0..samples_to_issue - 1 {
            let tenancy = ten_dist.generate();
            cache.add_tenancy(tenancy);
            dcsd_observed[cache.size as usize] += 1;
        }

        // prev_output = dcsd_observed.clone();
        cycles += 1;
    }
}


fn get_sum(input: &Vec<u64>) -> u128 {
    let mut sum: u128 = 0;

    input.iter().for_each(|&k| sum += k as u128);

    if sum == 0 {
        return 1;
    }
    return sum;
}


fn input_to_hashmap() -> (Generator, usize) {
    let mut rdr = csv::Reader::from_reader(io::stdin());
    let mut tenancy_dist: HashMap<u64, f64> = HashMap::new();
    let mut largest = 0;
    for result in rdr.records() {
        let record = result.unwrap();

        let tenancy = record.get(0).unwrap().parse().unwrap();
        let prob = f64::from_str(&record.get(1).unwrap()).unwrap();

        if tenancy > largest {
            largest = tenancy;
        }
        tenancy_dist.insert(tenancy, prob);
    }
    return (Generator::new(tenancy_dist.into_iter()), largest as usize);
}


fn write(output: Vec<u64>) {
    let sum = get_sum(&output);
    let mut wtr = csv::Writer::from_writer(io::stdout());
    let mut index: usize = 0;
    wtr.write_record(&["DCS", "probability"]).expect("cannot write");
    output.iter().for_each(|&key| {
        wtr.write_record(&[index.to_string(), ((key as f64) / sum as f64).to_string()]).expect("cannot write");
        // println!("{}: {}", index, key);
        index += 1;
    });
    println!("sum: {}", sum);
}


fn main() {
    // let test = input_to_hashmap();
    let (tenancy_dist, largest) = input_to_hashmap();
    let output = caching(tenancy_dist, 100, 0.005, largest);
    write(output);
}

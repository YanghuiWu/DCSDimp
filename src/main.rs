use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::ops::DerefMut;
use std::str::FromStr;

use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::prelude::ThreadRng;

/// Struct for weighted random sampling
struct Generator {
    random: RefCell<ThreadRng>,
    distribution: WeightedIndex<f64>,
    source: Vec<u64>,
}


impl Generator {
    /// Create a new sampler from an iterator of (item, weight) tuples
    fn new<T: Iterator<Item=(u64, f64)>>(t: T) -> Generator {
        let r = RefCell::new(rand::thread_rng());
        let vector: Vec<(u64, f64)> = t.into_iter().collect(); //Guarantees our index ordering.
        let distribution = WeightedIndex::new(vector.iter().map(|(_, weight)| *weight)).unwrap();
        let source = vector.into_iter().map(|(item, _)| item).collect();

        Generator {
            random: r,
            distribution,
            source,
        }
    }

    /// Sample an item using the weighted distribution
    fn generate(&self) -> u64 {
        let index = self
            .distribution
            .sample(self.random.borrow_mut().deref_mut());
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


    fn _get_size(&self) -> u64 {
        self.size
    }
}

/// Perform the caching simulation
fn caching(ten_dist: Generator, _cache_size: u64, _delta: f64, samples_to_issue: usize) -> Vec<u64> {
    let mut cache = Simulator::init();
    // let samples_to_issue: u64 = length as u64;
    // let mut prev_output: Vec<u64> = vec![0; samples_to_issue + 1];
    let mut dcsd_observed = vec![0; samples_to_issue + 1];
    let mut time = 0;
    //this part of code is for warmup cycles, but currently unused.
    if time == 0 {
        let tenancy = ten_dist.generate();
        cache.add_tenancy(tenancy);
        time += 1;
    }

    let mut cycles = 0;
    loop {
        println!("cycle: {}", cycles);
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
    let mut index: usize = 0;
    for k in input {
        sum += *k as u128;
        if index == input.len() {
            break;
        }
        index += 1;
    }
    if sum == 0 {
        return 1;
    }
    return sum;
}


fn input_to_hashmap() -> (HashMap<u64, f64>, usize) {
    // let mut rdr = csv::ReaderBuilder::new().from_reader(io::stdin());
    let mut rdr = csv::Reader::from_reader(io::stdin());
    let mut _result: HashMap<u64, f64> = HashMap::new();
    let mut largest = 0;
    for result in rdr.records() {
        let record = result.unwrap();
        // if record.get(0).unwrap().parse::<usize>().unwrap() > largest {
        //     largest = record.get(0).unwrap().parse().unwrap();
        // }
        // _result.insert(record.get(0).unwrap().parse().unwrap(), record.get(1).unwrap().parse().unwrap());

        let key = record.get(0).unwrap().parse().unwrap();
        let value = f64::from_str(&record.get(1).unwrap()).unwrap();

        if key > largest {
            largest = key;
        }
        _result.insert(key, value);
    }
    return (_result, largest as usize);
}


fn write(output: Vec<u64>) {
    let sum = get_sum(&output);
    let mut wtr = csv::Writer::from_writer(io::stdout());
    let mut index: usize = 0;
    wtr.write_record(&["DCS", "probability"]).expect("cannot write");
    for key in output {
        wtr.write_record(&[index.to_string(), ((key as f64) / sum as f64).to_string()]).expect("cannot write");
        index += 1;
    }
    println!("sum: {}", sum);
}


fn main() {
    let test = input_to_hashmap();
    let test_1 = caching(Generator::new(test.0.into_iter()), 10, 0.005, test.1);
    write(test_1);
}

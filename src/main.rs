/*
Copyright [2021] [l1npengtul<l1npengtul@protonmail.com>]

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
 */

use smallvec::SmallVec;
use dashmap::DashMap;
use std::{
    io::{Write, BufWriter},
    sync::Arc,
    time::Instant,
    fmt::{Display, Formatter},
    fs::File,
    cmp::Ordering
};
use twox_hash::RandomXxh3HashBuilder128;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

const PREALLOCATE_NUM: usize = 256;

#[derive(Clone, Debug)]
struct NumInfoStore {
    pub(crate) initial: u64,
    pub(crate) numbers: SmallVec<[u64; PREALLOCATE_NUM]>,
    pub(crate) steps: u64
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TotalInfoStore {
    pub(crate) idx: u64,
    pub(crate) numbers: SmallVec<[u64; PREALLOCATE_NUM]>,
    pub(crate) steps: u64
}

impl PartialOrd for TotalInfoStore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalInfoStore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl Display for NumInfoStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Steps: {}, {:?}", self.steps, self.numbers)
    }
}

fn main() {
    let start_int: u64 = {
        let args = std::env::args().nth(1).unwrap().parse::<u64>().unwrap();
        args
    };

    if start_int > usize::MAX as u64 {
        panic!("Exceeded usize!");
    }

    let output_path = {
        let args = std::env::args().nth(2).unwrap_or("output.txt".to_string());
        args
    };

    let threads = num_cpus::get();

    let threads = {
        let mut args = std::env::args().nth(3).unwrap_or(threads.to_string()).parse::<usize>().unwrap();
        if args > threads {
            args = threads;
        }
        args
    };

    println!("Calculating all Collatz Conjectures from 1 to {}, saving output to {}", start_int, output_path);

    let map: Arc<DashMap<u64, NumInfoStore, RandomXxh3HashBuilder128>> = Arc::new(DashMap::with_capacity_and_hasher(start_int as usize, twox_hash::RandomXxh3HashBuilder128::default()));

    let pool = rusty_pool::Builder::new()
        .core_size(threads)
        .max_size(threads)
        .build();

    println!("Pre-allocation done. Starting calculation with {} threads...", threads);
    let start_calculation_time = Instant::now();

    for collatz_to_calculate in 1..(start_int+1) {
        let map_ref = map.clone();
        let calc_int = collatz_to_calculate;
        pool.execute(move || {
            calculate(calc_int, map_ref)
        })
    }

    loop {
        if map.len() == start_int as usize {
            break;
        }
    }
    println!("Total Calculation Time: {}ms", start_calculation_time.elapsed().as_millis());
    println!("Saving to {}. Please wait...", output_path);

    let mut output_file = File::create(output_path).unwrap();
    let mut map_to_vec = Vec::new();
    for data in map.iter() {
        let mut dedup_data = data.numbers.clone();
        dedup_data.dedup();
        map_to_vec.push(
            TotalInfoStore {
                idx: data.initial,
                numbers: dedup_data,
                steps: data.steps,
            }
        )
    }

    map_to_vec.sort();
    let mut bufwriter = BufWriter::new(output_file);
    for (idx, item) in map_to_vec.iter().enumerate() {
        println!("Processing {}", idx);
        let to_write = format!("{}: Steps: {}, Numbers: {:?}\n", item.idx, item.steps, item.numbers);
        bufwriter.write(to_write.as_bytes());
    }
    println!("Flushing buffer...");
    bufwriter.flush().unwrap();
    println!("Done.")
}

fn calculate(start_int: u64, map: Arc<DashMap<u64, NumInfoStore, RandomXxh3HashBuilder128>>) {
    let init_int = start_int;
    let mut start_int = start_int;
    let mut steps = 0_u64;
    let mut steps_store: SmallVec<[u64; PREALLOCATE_NUM]> = SmallVec::with_capacity(PREALLOCATE_NUM);
    steps_store.push(init_int);

    if (start_int & 1) == 0 {
        let trailing = start_int.trailing_zeros();
        steps += trailing as u64;
        start_int = start_int >> trailing;
        steps_store.push(start_int);

    }

    loop {
        if start_int == 1 {
            break;
        }
        if map.contains_key(&start_int) {
            let mut info = match map.get(&start_int) {
                Some(i) => i.clone(),
                None => continue,
            };

            steps += info.steps;
            steps_store.append(&mut info.numbers);
            break;
        }
        if (start_int & 1) == 1 {
            start_int = (start_int << 2) - start_int + 1;
            steps += 1;
            steps_store.push(start_int);
        }
        else {
            start_int = start_int >> 1;
            steps += 1;
            steps_store.push(start_int);
        }
    }
    steps_store.truncate(steps_store.len());
    steps_store.shrink_to_fit();
    let num_info = NumInfoStore {
        initial: init_int,
        numbers: steps_store,
        steps,
    };

    map.insert(init_int, num_info);
}

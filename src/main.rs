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

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::time::Instant;

// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let start_int: u64 = {
        let args = std::env::args().nth(1).unwrap().parse::<u64>().unwrap();
        args
    };

    if start_int > usize::MAX as u64 {
        panic!("Exceeded usize!");
    }

    println!(
        "Calculating all Collatz Conjectures from 1 to {}",
        start_int
    );

    let mut total = 0;
    let now = Instant::now();
    let mut steps = Vec::with_capacity((start_int + 4) as usize);
    steps.push(0);
    steps.push(0);

    for num in 2..50000000 {
        let count = calculate(num, num, &steps);
        total += count;
        steps.push(count);
    }

    total += (50000000..(start_int + 1))
        .into_par_iter()
        .map(|f| calculate(f, 50000000, &steps))
        .sum::<u64>();

    println!("{}. Took {}ms", total, now.elapsed().as_millis());
}

#[inline(always)]
fn calculate(start_int: u64, bound: u64, map: &Vec<u64>) -> u64 {
    let mut start_int = start_int;
    let mut steps = 0_u64;

    if (start_int & 1) == 0 {
        let trailing = start_int.trailing_zeros();
        steps += trailing as u64;
        start_int = start_int >> trailing;
    }

    while start_int >= bound {
        if (start_int & 1) == 1 {
            start_int = ((start_int << 2) - start_int + 1) >> 1;
            steps += 2;
        } else {
            start_int = start_int >> 1;
            steps += 1;
        }
        if start_int == 1 {
            break;
        }
    }

    steps + map[start_int as usize]
}

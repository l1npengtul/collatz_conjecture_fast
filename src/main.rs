fn main() {
    let mut start_int: u128 = {
        let args = std::env::args().nth(1).unwrap().parse::<u128>().unwrap();
        args
    };
    let mut steps: usize = 0;
    loop {
        if (start_int & 1) == 1 {
            start_int = (start_int << 2) - start_int + 1;
            steps += 1;
        }
        else {
            start_int = start_int >> 1;
            steps += 1;
        }
        if start_int == 1 {
            break;
        }
    }
    println!("{:?}", steps)
}

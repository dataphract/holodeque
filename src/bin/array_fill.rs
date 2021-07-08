use std::time::Instant;

use holodeque::SliceDeque;

fn main() {
    let mut buf: Vec<u64> = Vec::with_capacity(8 * 1024 * 1024);
    buf.resize(buf.capacity(), 0);

    let start = Instant::now();

    let mut deque = SliceDeque::new_in(&mut buf);

    for i in 0..deque.capacity() {
        deque.push_back(i as u64).unwrap();
    }

    let elapsed = start.elapsed();

    println!("elapsed: {}μs", elapsed.as_micros());
}

#[macro_use]
extern crate bencher;

extern crate queries_core;

use bencher::Bencher;
use queries_core::preprocess;

fn prep_short(bench: &mut Bencher) {
    bench.iter(|| {
        preprocess("Shut up");
    })
}

fn prep_medium(bench: &mut Bencher) {
    bench.iter(|| {
        preprocess("Book me a uber right now")
    })
}

fn prep_long(bench: &mut Bencher) {
    bench.iter(|| {
        preprocess("Book me a table for four people at a Café nearby for tomorrow night at 8pm")
    })
}

fn prep_longer(bench: &mut Bencher) {
    bench.iter(|| {
        preprocess("Book me a table for four people at a Café nearby for tomorrow night at 8pm that my parents will like because they have terrible taste and don't like exotic food because they fear it will be too spicy")
    })
}

benchmark_group!(benches, prep_short, prep_medium, prep_long, prep_longer);
benchmark_main!(benches);

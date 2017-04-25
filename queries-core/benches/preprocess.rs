#[macro_use]
extern crate bencher;
extern crate queries_core;

use bencher::Bencher;

fn prep_short(bench: &mut Bencher) {
    let text = "Shut up";

    bench.iter(|| { let _ = queries_core::preprocess(text); })
}

fn prep_medium(bench: &mut Bencher) {
    let text = "Book me a uber right now";

    bench.iter(|| { let _ = queries_core::preprocess(text); })
}

fn prep_long(bench: &mut Bencher) {
    let text = "Book me a table for four people at a Café nearby for tomorrow night at 8pm";

    bench.iter(|| { let _ = queries_core::preprocess(text); })
}

fn prep_longer(bench: &mut Bencher) {
    let text = "Book me a table for four people at a Café nearby for tomorrow night at 8pm \
    that my parents will like because they have terrible taste and don't like \
    exotic food because they fear it will be too spicy";

    bench.iter(|| { let _ = queries_core::preprocess(text); })
}

benchmark_group!(benches, prep_short, prep_medium, prep_long, prep_longer);
benchmark_main!(benches);

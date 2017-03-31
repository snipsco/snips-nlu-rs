#[macro_use]
extern crate bencher;
extern crate queries_core;

use bencher::Bencher;

fn prep_short(bench: &mut Bencher) {
    let text = "Shut up";
    let entities = "[]";

    bench.iter(|| { let _ = queries_core::preprocess(text, entities); })
}

fn prep_medium(bench: &mut Bencher) {
    let text = "Book me a uber right now";
    let entities = r#"[{"end_index": 24, "value": "right now", "start_index": 15, "entity": "%TIME%"}]"#;

    bench.iter(|| { let _ = queries_core::preprocess(text, entities); })
}

fn prep_long(bench: &mut Bencher) {
    let text = "Book me a table for four people at a Café nearby for tomorrow night at 8pm";
    let entities = r#"[{"end_index": 24, "value": "four", "start_index": 20, "entity": "%NUMBER%"}, {"end_index": 74, "value": "tomorrow night at 8pm", "start_index": 53, "entity": "%TIME%"}]"#;

    bench.iter(|| { let _ = queries_core::preprocess(text, entities); })
}

fn prep_longer(bench: &mut Bencher) {
    let text = "Book me a table for four people at a Café nearby for tomorrow night at 8pm \
    that my parents will like because they have terrible taste and don't like \
    exotic food because they fear it will be too spicy";
    let entities = r#"[{"end_index": 24, "value": "four", "start_index": 20, "entity": "%NUMBER%"}, {"end_index": 74, "value": "tomorrow night at 8pm", "start_index": 53, "entity": "%TIME%"}]"#;

    bench.iter(|| { let _ = queries_core::preprocess(text, entities); })
}

benchmark_group!(benches, prep_short, prep_medium, prep_long, prep_longer);
benchmark_main!(benches);

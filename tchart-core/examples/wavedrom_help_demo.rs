//! One-off demo to regenerate the WaveJSON snippets shown in the help page.
//!
//! Usage: `cargo run --example wavedrom_help_demo -p tchart-core`.
//! Each section prints a header followed by the JSON to stdout. Pipe through
//! `wavedrom-cli` to obtain the rendered SVG inlined into `help/`.

use tchart_core::parser::parse;
use tchart_core::wavedrom::to_wavejson;

const SAMPLES: &[(&str, &str)] = &[
    (
        "gap",
        "@title 連続性の断絶
sig1   ~_~_:~_~_
sig2   ====:====
",
    ),
    (
        "bus_x_transitions",
        "@title バス値の切替
clk    ~_~_~_~_
data   ==A=X=B=X=C
",
    ),
    (
        "arrow",
        "@title 信号間にまたがる矢印
clk    ~_~_~_~_
req    _@{request}~~~~~~_
ack    ___@{ack_received}~~~~_
done   ______@{complete}~_
@-> (@{request}, @{ack_received}) ack
@-> (@{ack_received}, @{complete}) done
",
    ),
];

fn main() {
    for (name, tcml) in SAMPLES {
        let document = parse(tcml).expect("parse");
        let (json, warnings) = to_wavejson(&document);
        for warning in &warnings {
            eprintln!("[{name}] {warning}");
        }
        println!("=== {name} ===");
        println!("{json}");
    }
}

use std::fs;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

struct Args {
    iterations: usize,
    benchmark: String,
}

#[derive(serde::Deserialize, Debug)]
struct Benchmark {
    name: String,
    html: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();
    let args = Args {
        iterations: args.value_from_str("--iterations")?,
        benchmark: args.value_from_str("--benchmark")?,
    };

    let benchmarks_str =
        fs::read_to_string("../benchmarks/benchmarks.json").expect("Failed to load benchmarks");
    let benchmarks: Vec<Benchmark> =
        serde_json::from_str(&benchmarks_str).expect("Failed to load benchmarks");
    if let Some(benchmark) = benchmarks.iter().find(|x| x.name == args.benchmark) {
        let mut output = Vec::with_capacity(
            (benchmark.html.len() as f64 * 1.5)
                .min(usize::MAX as f64)
                .round() as usize,
        );
        for _ in 0..args.iterations {
            #[cfg(feature = "dhat-heap")]
            let _profiler = dhat::Profiler::new_heap();
            css_inline::blocking::inline_to(&benchmark.html, &mut output).expect("Inlining failed");
            output.clear();
        }
    } else {
        panic!("Can not find benchmark: {}", &args.benchmark)
    }

    Ok(())
}

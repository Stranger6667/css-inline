# css_inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css_inline` is a high-performance library for inlining CSS into HTML 'style' attributes.

## Performance

This library uses components from Mozilla's Servo project for CSS parsing and matching.
Performance benchmarks show 3-9x faster execution than `tijsverkoyen/css-to-inline-styles`.

The table below shows benchmark results comparing `css_inline` with `tijsverkoyen/css-to-inline-styles` on typical HTML documents:

|                   | Size    | `css_inline 0.15.0` | `tijsverkoyen/css-to-inline-styles 2.2.7` | Speedup |
|-------------------|---------|---------------------|-------------------------------------------|---------|
| Simple            | 230 B   | 5.99 µs             | 28.06 µs                                  | **4.68x** |
| Realistic email 1 | 8.58 KB | 102.25 µs           | 313.31 µs                                 | **3.06x** |
| Realistic email 2 | 4.3 KB  | 71.98 µs            | 655.43 µs                                 | **9.10x** |
| GitHub Page†      | 1.81 MB | 163.80 ms           | 8.22 ms*                                  | N/A |

> † The GitHub page benchmark uses modern CSS that `tijsverkoyen/css-to-inline-styles` cannot process, resulting in skipped styles and an invalid comparison.

Please refer to the `benchmarks/InlineBench.php` file to review the benchmark code.
The results displayed above were measured using stable `rustc 1.88` on PHP `8.4.10`.

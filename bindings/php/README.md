# css_inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css_inline` is a high-performance library for inlining CSS into HTML 'style' attributes.

## Performance

This library uses components from Mozilla's Servo project for CSS parsing and matching.
Performance benchmarks show significant speed improvements over other popular PHP CSS inlining libraries.

|                   | Size    | `css_inline 0.15.0` | `css-to-inline-styles 2.3.0` | `emogrifier 7.3.0`      |
|-------------------|---------|---------------------|------------------------------|-------------------------|
| Simple            | 230 B   | 5.99 µs             | 28.06 µs (**4.68x**)         | 137.85 µs (**23.01x**)  |
| Realistic email 1 | 8.58 KB | 102.25 µs           | 313.31 µs (**3.06x**)        | 637.75 µs (**6.24x**)   |
| Realistic email 2 | 4.3 KB  | 71.98 µs            | 655.43 µs (**9.10x**)        | 2.32 ms (**32.21x**)    |
| GitHub Page†      | 1.81 MB | 163.80 ms           | ERROR                        | ERROR                   |

† The GitHub page benchmark contains complex modern CSS that neither `css-to-inline-styles` nor `emogrifier` can process and didn't finish a single iteration in >10 minutes.

Please refer to the `benchmarks/InlineBench.php` file to review the benchmark code.
The results displayed above were measured using stable `rustc 1.88` on PHP `8.4.10`.

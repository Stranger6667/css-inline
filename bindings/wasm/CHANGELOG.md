# Changelog

## [Unreleased]

## [0.7.0] - 2021-06-09

### Fixed

- Ignored selectors specificity. [#108](https://github.com/Stranger6667/css-inline/issues/108)

## [0.6.1] - 2020-12-07

### Fixed

- Compatibility with the new `cssparser` crate version.

### Performance

- Avoid string allocations during converting `InlineError` to `JsValue`.

## [0.6.0] - 2020-11-02

### Changed

- Links to remote stylesheets are deduplicated now.

### Performance

- Use `Formatter.write_str` instead of `write!` macro in the `Display` trait implementation for `InlineError`. [#85](https://github.com/Stranger6667/css-inline/issues/85)
- Use `Cow` for error messages. [#87](https://github.com/Stranger6667/css-inline/issues/87)

## [0.5.0] - 2020-08-07

### Performance

- Avoid string allocation in `get_full_url`

## [0.4.0] - 2020-07-13

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/wasm-v0.7.0...HEAD
[0.7.0]: https://github.com/Stranger6667/css-inline/compare/wasm-v0.6.1...wasm-v0.7.0
[0.6.1]: https://github.com/Stranger6667/css-inline/compare/wasm-v0.6.0...wasm-v0.6.1
[0.6.0]: https://github.com/Stranger6667/css-inline/compare/wasm-v0.5.0...wasm-v0.6.0
[0.5.0]: https://github.com/Stranger6667/css-inline/compare/wasm-v0.4.1...wasm-v0.5.0
[0.4.1]: https://github.com/Stranger6667/css-inline/compare/wasm-v0.4.0...wasm-v0.4.1

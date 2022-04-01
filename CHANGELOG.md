# Changelog

## [Unreleased]

### Fixed

- Not respecting specificity in case of inlining overlapping rules like `padding` and `padding-top`. [#142](https://github.com/Stranger6667/css-inline/issues/142)

### Performance

- Pre-allocate more memory for output HTML to avoid resizing.

## [0.8.0] - 2022-01-09

### Added

- Separate `InlineError::MissingStyleSheet` error variant to improve debugging experience. [#124](https://github.com/Stranger6667/css-inline/issues/124)

## [0.7.6] - 2022-01-08

### Fixed

- Invalid handling of double-quoted property values like in `font-family: "Open Sans"`. [#129](https://github.com/Stranger6667/css-inline/issues/129)

### Performance

- Use `std::fs::read_to_string` in CLI to avoid over/under allocating of the input buffer.

## [0.7.5] - 2021-07-24

### Fixed

- Panic on invalid URLs for remote stylesheets.

## [0.7.4] - 2021-07-06

### Changed

- Update `rayon` to `1.5`.

### Performance

- Optimize loading of external files.

## [0.7.3] - 2021-06-24

### Performance

- Avoid allocations in error formatting.

## [0.7.2] - 2021-06-22

### Fixed

- Incorrect override of exiting `style` attribute values. [#113](https://github.com/Stranger6667/css-inline/issues/113)

### Performance

- Use specialized `to_string` implementation on `&&str`.
- Use `ahash`.

## [0.7.1] - 2021-06-10

### Fixed

- Ignored `style` tags when the document contains multiple of them and the `remove_style_tags: true` config option is used. [#110](https://github.com/Stranger6667/css-inline/issues/110)

## [0.7.0] - 2021-06-09

### Fixed

- Ignored selectors specificity. [#108](https://github.com/Stranger6667/css-inline/issues/108)

## [0.6.1] - 2020-12-07

### Fixed

- Compatibility with the new `cssparser` crate version.

### Performance

- Avoid string allocations during converting `ParseError` to `InlineError`.

## [0.6.0] - 2020-11-02

### Changed

- Links to remote stylesheets are deduplicated now.

### Fixed

- Wrong inlined file prefixes handling in CLI. [#89](https://github.com/Stranger6667/css-inline/issues/89)

### Performance

- Use `Formatter.write_str` instead of `write!` macro in the `Display` trait implementation for `InlineError`. [#85](https://github.com/Stranger6667/css-inline/issues/85)
- Use `Cow` for error messages. [#87](https://github.com/Stranger6667/css-inline/issues/87)

## [0.5.0] - 2020-08-07

### Added

- `CSSInliner::options()` that implements the Builder pattern. [#71](https://github.com/Stranger6667/css-inline/issues/71)

### Changed

- Restrict visibility of items in `parser.rs`

### Performance

- Avoid string allocation in `get_full_url`

## [0.4.0] - 2020-07-13

### Added

- Option to disable processing of "style" tags. [#45](https://github.com/Stranger6667/css-inline/issues/45)
- Option to inline additional CSS. [#45](https://github.com/Stranger6667/css-inline/issues/45)

### Changed

- Switch from `openssl` to `rustls` in `attohttpc` dependency. [#49](https://github.com/Stranger6667/css-inline/issues/49)

### Performance

- Use `codegen-units=1` and `lto=fat`.
- Reduce the number of allocations in CLI.
- Avoid CLI output formatting when it is not needed.

## [0.3.3] - 2020-07-07

### Performance

- Pre-allocate the output vector.
- Minor improvement for creating new files via CLI.
- Reduce the average number of allocations during styles merge by a factor of 5.5x.

## [0.3.2] - 2020-06-27

### Changed

- Remove debug symbols from the release build

### Performance

- Reduce the number of `String` allocations.
- Avoid `BTreeMap::insert` when `style` attribute already exists

## [0.3.1] - 2020-06-25

### Changed

- Fix links in docs

## [0.3.0] - 2020-06-25

### Added

- Command Line Interface. [#33](https://github.com/Stranger6667/css-inline/issues/33)

## [0.2.0] - 2020-06-25

### Added

- `CSSInliner` and customization options. [#9](https://github.com/Stranger6667/css-inline/issues/9)
- Option to remove "style" tags (`remove_style_tags`). Disabled by default. [#11](https://github.com/Stranger6667/css-inline/issues/11)
- `CSSInliner::compact()` constructor for producing smaller HTML output.
- `CSSInliner.inline_to` that writes the output to a generic writer. [#24](https://github.com/Stranger6667/css-inline/issues/24)
- Implement `Error` for `InlineError`.
- Loading external stylesheets. [#8](https://github.com/Stranger6667/css-inline/issues/8)
- Option to control whether remote stylesheets should be loaded (`load_remote_stylesheets`). Enabled by default.

### Changed

- Improved error messages. [#27](https://github.com/Stranger6667/css-inline/issues/27)
- Skip selectors that can't be parsed.

### Fixed

- Ignore `@media` queries since they can not be inlined. [#7](https://github.com/Stranger6667/css-inline/issues/7)
- Panic in cases when styles are applied to the currently processed "link" tags.

### Performance

- Improve performance for merging new styles in existing "style" attributes.

## 0.1.0 - 2020-06-22

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/rust-v0.8.0...HEAD
[0.8.0]: https://github.com/Stranger6667/css-inline/compare/rust-v0.7.6...rust-v0.8.0
[0.7.6]: https://github.com/Stranger6667/css-inline/compare/rust-v0.7.5...rust-v0.7.6
[0.7.5]: https://github.com/Stranger6667/css-inline/compare/rust-v0.7.4...rust-v0.7.5
[0.7.4]: https://github.com/Stranger6667/css-inline/compare/rust-v0.7.3...rust-v0.7.4
[0.7.3]: https://github.com/Stranger6667/css-inline/compare/rust-v0.7.2...rust-v0.7.3
[0.7.2]: https://github.com/Stranger6667/css-inline/compare/rust-v0.7.1...rust-v0.7.2
[0.7.1]: https://github.com/Stranger6667/css-inline/compare/rust-v0.7.0...rust-v0.7.1
[0.7.0]: https://github.com/Stranger6667/css-inline/compare/rust-v0.6.1...rust-v0.7.0
[0.6.1]: https://github.com/Stranger6667/css-inline/compare/rust-v0.6.0...rust-v0.6.1
[0.6.0]: https://github.com/Stranger6667/css-inline/compare/rust-v0.5.0...rust-v0.6.0
[0.5.0]: https://github.com/Stranger6667/css-inline/compare/rust-v0.4.0...rust-v0.5.0
[0.4.0]: https://github.com/Stranger6667/css-inline/compare/0.3.3...rust-v0.4.0
[0.3.3]: https://github.com/Stranger6667/css-inline/compare/0.3.2...0.3.3
[0.3.2]: https://github.com/Stranger6667/css-inline/compare/0.3.1...0.3.2
[0.3.1]: https://github.com/Stranger6667/css-inline/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/Stranger6667/css-inline/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/Stranger6667/css-inline/compare/0.1.0...0.2.0

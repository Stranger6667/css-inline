# Changelog

## [Unreleased]

## [0.8.7] - 2023-01-30

### Added

- Distribute typing information. [#167](https://github.com/Stranger6667/css-inline/issues/167)

### Changed

- Update `PyO3` to `0.18.0`.

## [0.8.6] - 2022-11-10

### Added

- Support for the `file://` URI scheme in `base_url`. [#171](https://github.com/Stranger6667/css-inline/issues/171)

## [0.8.5] - 2022-11-02

### Added

- `data-css-inline="ignore"` attribute to ignore CSS inlining. [#10](https://github.com/Stranger6667/css-inline/issues/10)
- Python 3.11 support.

## [0.8.4] - 2022-10-20

### Fixed

- Ignoring selectors' specificity when applying declarations from different qualified rules. [#148](https://github.com/Stranger6667/css-inline/issues/148)

### Changed

- Update `PyO3` to `0.17.2`.

### Removed

- Python 3.6 support.

## [0.8.3] - 2022-07-21

### Fixed

- `!important` rules not overriding inlined styles. [#152](https://github.com/Stranger6667/css-inline/issues/152)

## [0.8.2] - 2022-04-01

### Fixed

- Not respecting specificity in case of inlining overlapping rules like `padding` and `padding-top`. [#142](https://github.com/Stranger6667/css-inline/issues/142)

## [0.8.1] - 2022-01-11

### Added

- Universal macOS wheels supporting CPython 3.6 & 3.7 on Apple Silicon.

## [0.8.0] - 2022-01-10

### Added

- Include missing stylesheet path to `InlineError` message. [#124](https://github.com/Stranger6667/css-inline/issues/124)
- Build wheels for more platforms, including CPython on Apple Silicon, and PyPy on x86_64. [#102](https://github.com/Stranger6667/css-inline/issues/102), [#131](https://github.com/Stranger6667/css-inline/issues/131)

## [0.7.8] - 2022-01-08

### Fixed

- Invalid handling of double-quoted property values like in `font-family: "Open Sans"`. [#129](https://github.com/Stranger6667/css-inline/issues/129)

## [0.7.7] - 2022-01-07

### Added

- Python 3.10 builds.

### Changed

- Update `PyO3` to `0.15.1`.

## [0.7.6] - 2021-08-06

### Fixed

- Docs: Link to the project homepage in `setup.py`.

## [0.7.5] - 2021-07-24

### Fixed

- Panic on invalid URLs for remote stylesheets.

## [0.7.4] - 2021-07-06

### Changed

- Update `PyO3` to `0.14.1`.

### Performance

- Optimize loading of external files.

## [0.7.3] - 2021-06-24

### Performance

- Improve performance for error handling.

## [0.7.2] - 2021-06-22

### Fixed

- Incorrect override of exiting `style` attribute values. [#113](https://github.com/Stranger6667/css-inline/issues/113)

### Performance

- Minor performance improvements

## [0.7.1] - 2021-06-10

### Fixed

- Ignored `style` tags when the document contains multiple of them and the `remove_style_tags: true` config option is used. [#110](https://github.com/Stranger6667/css-inline/issues/110)

## [0.7.0] - 2021-06-09

### Fixed

- Ignored selectors specificity. [#108](https://github.com/Stranger6667/css-inline/issues/108)

### Changed

- Upgrade `Pyo3` to `0.13`.

## [0.6.2] - 2021-01-28

### Fixed

- Source code distribution. It was missing the source code for the underlying Rust crate and were leading to
  a build error during `pip install css-inline` on platforms that we don't have wheels for.
  [#99](https://github.com/Stranger6667/css-inline/issues/99)

## [0.6.1] - 2020-12-07

### Added

- Python 3.9 support.

### Fixed

- Compatibility with the new `cssparser` crate version.

### Performance

- Avoid string allocations during converting `ParseError` to `InlineError`.

## [0.6.0] - 2020-11-02

### Changed

- Links to remote stylesheets are deduplicated now.
- Upgrade `Pyo3` to `0.12`.

### Performance

- Avoid setting module docstring twice
- Use `Cow` for error messages. [#87](https://github.com/Stranger6667/css-inline/issues/87)

## [0.5.0] - 2020-08-07

### Performance

- Avoid string allocation in `get_full_url`

## [0.4.0] - 2020-07-13

### Added

- Option to disable processing of "style" tags. [#45](https://github.com/Stranger6667/css-inline/issues/45)
- Option to inline additional CSS. [#45](https://github.com/Stranger6667/css-inline/issues/45)

### Changed

- Switch from `openssl` to `rustls` in `attohttpc` dependency. [#49](https://github.com/Stranger6667/css-inline/issues/49)

### Performance

- Use `ToString` trait during error handling to avoid using a formatter.

## [0.3.2] - 2020-07-09

### Fixed

- `CSSInliner` signature detection in PyCharm.

## [0.3.1] - 2020-07-07

### Changed

- Upgrade `Pyo3` to `0.11`. [#40](https://github.com/Stranger6667/css-inline/issues/40)

### Performance

- Pre-allocate the output vector.
- Reduce the average number of allocations during styles merge by a factor of 5.5x.

## [0.3.0] - 2020-06-27

### Changed

- Remove debug symbols from the release build

### Performance

- Various performance improvements

## [0.2.0] - 2020-06-25

### Added

- Loading external stylesheets. [#8](https://github.com/Stranger6667/css-inline/issues/8)
- Option to control whether remote stylesheets should be loaded (`load_remote_stylesheets`). Enabled by default.

### Changed

- Skip selectors that can't be parsed.
- Validate `base_url` to be a valid URL.

### Fixed

- Panic in cases when styles are applied to the currently processed "link" tags.

## 0.1.0 - 2020-06-24

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/python-v0.8.7...HEAD
[0.8.7]: https://github.com/Stranger6667/css-inline/compare/python-v0.8.6...python-v0.8.7
[0.8.6]: https://github.com/Stranger6667/css-inline/compare/python-v0.8.5...python-v0.8.6
[0.8.5]: https://github.com/Stranger6667/css-inline/compare/python-v0.8.4...python-v0.8.5
[0.8.4]: https://github.com/Stranger6667/css-inline/compare/python-v0.8.3...python-v0.8.4
[0.8.3]: https://github.com/Stranger6667/css-inline/compare/python-v0.8.2...python-v0.8.3
[0.8.2]: https://github.com/Stranger6667/css-inline/compare/python-v0.8.1...python-v0.8.2
[0.8.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.8.0...python-v0.8.1
[0.8.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.8...python-v0.8.0
[0.7.8]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.7...python-v0.7.8
[0.7.7]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.6...python-v0.7.7
[0.7.6]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.5...python-v0.7.6
[0.7.5]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.4...python-v0.7.5
[0.7.4]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.3...python-v0.7.4
[0.7.3]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.2...python-v0.7.3
[0.7.2]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.1...python-v0.7.2
[0.7.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.7.0...python-v0.7.1
[0.7.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.6.2...python-v0.7.0
[0.6.2]: https://github.com/Stranger6667/css-inline/compare/python-v0.6.1...python-v0.6.2
[0.6.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.6.0...python-v0.6.1
[0.6.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.5.0...python-v0.6.0
[0.5.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.4.0...python-v0.5.0
[0.4.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.3.2...python-v0.4.0
[0.3.2]: https://github.com/Stranger6667/css-inline/compare/python-v0.3.1...python-v0.3.2
[0.3.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.3.0...python-v0.3.1
[0.3.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.2.0...python-v0.3.0
[0.2.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.1.0...python-v0.2.0

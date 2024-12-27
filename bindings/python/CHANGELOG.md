# Changelog

## [Unreleased]

## [0.14.6] - 2024-12-27

### Fixed

- Packaging issue.

## [0.14.5] - 2024-12-27

### Fixed

- Packaging issue.

## [0.14.4] - 2024-12-27

### Added

- Support & build for PyOdide. [#411](https://github.com/Stranger6667/css-inline/pull/411)

### Changed

- Bump MSRV to `1.71.1`.

## [0.14.3] - 2024-11-14

### Fixed

- Prioritize `!important` rules when computing element styles. [#398](https://github.com/Stranger6667/css-inline/pull/398)

## [0.14.2] - 2024-11-11

### Changed

- Update `PyO3` to `0.22.0`.
- Bump MSRV to `1.70`.

### Fixed

- Replace double quotes when merging styles. [#392](https://github.com/Stranger6667/css-inline/issues/392)

## [0.14.1] - 2024-04-27

### Fixed

- Precedence of element styles over other styles. [#364](https://github.com/Stranger6667/css-inline/issues/364)

## [0.14.0] - 2024-04-01

### Added

- External stylesheet caching. [#314](https://github.com/Stranger6667/css-inline/issues/314)
- Inlining to HTML fragments. [#335](https://github.com/Stranger6667/css-inline/issues/335)

### Changed

- Update `html5ever` to `0.27`.
- Update `PyO3` to `0.21.0`.

## [0.13.0] - 2024-01-19

### Added

- Support for the `data-css-inline="keep"` attribute to enforce keeping the `style` tag.

### Fixed

- Lookups for previous / next siblings, affecting selectors like `nth-child`. [#324](https://github.com/Stranger6667/css-inline/issues/324)

### Performance

- Avoid using binary search on attributes.

## [0.12.0] - 2023-12-28

### Changed

- Display stylesheet location in network-related errors.

### Performance

- Optimize serialization of attributes and text nodes.

## [0.11.2] - 2023-12-09

### Performance

- Avoid iterating over non-Element nodes.
- Reuse caches for nth index selectors.

## [0.11.1] - 2023-12-09

### Added

- Python 3.12 support.
- Build wheels for Python 3.10 on PyPy.

### Changed

- Update `indexmap` to `2.1`.
- Update `cssparser` to `0.31.2`.
- Update `selectors` to `0.25`.
- Bump MSRV to `1.65`.
- Update `PyO3` to `0.20.0`.
- Update `built` to `0.7.1`.
- Bump `manylinux` version for `aarch64` wheels to `2_24`.

### Fixed

- Replace double quotes in all property values.

### Performance

- Avoid allocation when replacing double quotes in property values.

## [0.11.0] - 2023-09-26

### Added

- The `inline_style_tags` option to control whether inlining from "style" tags should be performed. [#253](https://github.com/Stranger6667/css-inline/issues/253)

### Performance

- Reuse existing attributes when creating an element during parsing.

### Changed

- Bump MSRV to `1.63`.

## [0.10.5] - 2023-08-30

### Performance

- Pre-allocate space during serialization.
- Optimized `class` attribute handling: up to 25% faster for extensive class-dependent selectors.
- Fast-path class check for shorter class attribute values.
- Use a Bloom filter to detect if an element has no given class.
- Avoid allocating a vector during selectors compilation.
- Use `FxHasher` in more cases.

### Changed

- Drop usage of `memchr`.
- Bump MSRV to `1.62.1`.

## [0.10.4] - 2023-08-12

### Fixed

- Applying new styles only to the first matching tag during styles merging. [#224](https://github.com/Stranger6667/css-inline/issues/224)

### Performance

- Fix under-allocating storage for intermediate CSS styles.
- Perform CSS inlining as late as possible to avoid intermediate allocations. [#220](https://github.com/Stranger6667/css-inline/issues/220)

## [0.10.3] - 2023-07-01

### Performance

- Optimized HTML serialization for a performance boost of up to 25%.

## [0.10.2] - 2023-06-25

### Changed

- Standardized the formatting of CSS declarations: now consistently using `: ` separator between properties and values.

### Performance

- Various performance improvements.

## [0.10.1] - 2023-06-18

### Performance

- Use a simpler way for HTML tree traversal.
- Avoid hashing in some cases.

## [0.10.0] - 2023-06-16

### Added

- `keep_link_tags` configuration option.

### Changed

- Replace `remove_style_tags` with `keep_style_tags`.

### Fixed

- **SECURITY**: Passing unescaped strings in attribute values introduced in [#184](https://github.com/Stranger6667/css-inline/issues/184).
  Previously, escaped values became unescaped on the serialization step.

### Removed

- The `inline_style_tags` configuration option.

## [0.9.0] - 2023-06-10

### Fixed

- Serialize all HTML attributes without escaping. [#184](https://github.com/Stranger6667/css-inline/issues/184)

### Changed

- Update `PyO3` to `0.19.0`.

### Performance

- 30-50% average performance improvement due switch to a custom-built HTML tree representation and serializer.

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

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/python-v0.14.6...HEAD
[0.14.6]: https://github.com/Stranger6667/css-inline/compare/python-v0.14.5...python-v0.14.6
[0.14.5]: https://github.com/Stranger6667/css-inline/compare/python-v0.14.4...python-v0.14.5
[0.14.4]: https://github.com/Stranger6667/css-inline/compare/python-v0.14.3...python-v0.14.4
[0.14.3]: https://github.com/Stranger6667/css-inline/compare/python-v0.14.2...python-v0.14.3
[0.14.2]: https://github.com/Stranger6667/css-inline/compare/python-v0.14.1...python-v0.14.2
[0.14.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.14.0...python-v0.14.1
[0.14.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.13.0...python-v0.14.0
[0.13.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.12.0...python-v0.13.0
[0.12.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.11.2...python-v0.12.0
[0.11.2]: https://github.com/Stranger6667/css-inline/compare/python-v0.11.1...python-v0.11.2
[0.11.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.11.0...python-v0.11.1
[0.11.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.10.5...python-v0.11.0
[0.10.5]: https://github.com/Stranger6667/css-inline/compare/python-v0.10.4...python-v0.10.5
[0.10.4]: https://github.com/Stranger6667/css-inline/compare/python-v0.10.3...python-v0.10.4
[0.10.3]: https://github.com/Stranger6667/css-inline/compare/python-v0.10.2...python-v0.10.3
[0.10.2]: https://github.com/Stranger6667/css-inline/compare/python-v0.10.1...python-v0.10.2
[0.10.1]: https://github.com/Stranger6667/css-inline/compare/python-v0.10.0...python-v0.10.1
[0.10.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.9.0...python-v0.10.0
[0.9.0]: https://github.com/Stranger6667/css-inline/compare/python-v0.8.7...python-v0.9.0
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

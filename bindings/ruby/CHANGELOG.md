# Changelog

## [Unreleased]

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

### Changed

- Update `indexmap` to `2.1`.
- Update `cssparser` to `0.31.2`.
- Update `selectors` to `0.25`.
- Bump MSRV to `1.65`.

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

### Changed

- Set the default value for `preallocate_node_capacity` to `32` to match other the default value in other bindings.

## [0.10.4] - 2023-08-12

### Changed

- Update `magnus` to `0.6`.

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

## 0.10.0 - 2023-06-17

- Initial public release

[Unreleased]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.12.0...HEAD
[0.12.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.11.2...ruby-v0.12.0
[0.11.2]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.11.1...ruby-v0.11.2
[0.11.1]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.11.0...ruby-v0.11.1
[0.11.0]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.5...ruby-v0.11.0
[0.10.5]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.4...ruby-v0.10.5
[0.10.4]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.3...ruby-v0.10.4
[0.10.3]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.2...ruby-v0.10.3
[0.10.2]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.1...ruby-v0.10.2
[0.10.1]: https://github.com/Stranger6667/css-inline/compare/ruby-v0.10.0...ruby-v0.10.1

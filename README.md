# css-inline

[![ci](https://github.com/Stranger6667/css-inline/workflows/ci/badge.svg)](https://github.com/Stranger6667/css-inline/actions)
[![Crates.io](https://img.shields.io/crates/v/css-inline.svg)](https://crates.io/crates/css-inline)
[![docs.rs](https://docs.rs/css-inline/badge.svg?version=0.1.0)](https://docs.rs/css-inline/0.1.0/css-inline/)

A crate for inlining CSS into HTML documents. When you send HTML emails you need to use "style" attributes instead of "style" tags.

For example, this HTML:

```html
<html>
    <head>
        <title>Test</title>
        <style>
            h1, h2 { color:blue; }
            strong { text-decoration:none }
            p { font-size:2px }
            p.footer { font-size: 1px}
        </style>
    </head>
    <body>
        <h1>Big Text</h1>
        <p>
            <strong>Solid</strong>
        </p>
        <p class="footer">Foot notes</p>
    </body>
</html>
```

Will be turned into this:

```html
<html>
    <head>
        <title>Test</title>
    </head>
    <body>
        <h1 style="color:blue;">Big Text</h1>
        <p style="font-size:2px;">
            <strong style="text-decoration:none;">Solid</strong>
        </p>
        <p style="font-size:1px;">Foot notes</p>
    </body>
</html>
```

To use it in your project add the following line to your `dependencies` section in project's `Cargo.toml` file:

```toml
cssinline = "0.1"
```

## Example:

```rust
use css_inline;

const HTML: &str = r#"<html>
<head>
    <title>Test</title>
    <style>
        h1, h2 { color:blue; }
        strong { text-decoration:none }
        p { font-size:2px }
        p.footer { font-size: 1px}
    </style>
</head>
<body>
    <h1>Big Text</h1>
    <p>
        <strong>Solid</strong>
    </p>
    <p class="footer">Foot notes</p>
</body>
</html>"#;

fn main() -> Result<(), css_inline::InlineError> {
   let inlined = css_inline::inline(HTML)?;
   // Do something with inlined HTML, e.g. send an email
   Ok(())
}
```

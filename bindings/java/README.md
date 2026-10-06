# css-inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="github packages" src="https://img.shields.io/badge/github%20packages-css--inline-66c2a5?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/packages)

Java bindings for `css-inline`, a library that inlines CSS into HTML `style` attributes. Use it to prepare HTML emails or to embed HTML into third-party web pages.

It turns this HTML:

```html
<html>
  <head>
    <style>h1 { color:blue; }</style>
  </head>
  <body>
    <h1>Big Text</h1>
  </body>
</html>
```

into:

```html
<html>
  <head></head>
  <body>
    <h1 style="color:blue;">Big Text</h1>
  </body>
</html>
```

## Features

- Parses HTML and CSS with Mozilla Servo components (`html5ever`, `cssparser`, `selectors`)
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Loads external stylesheets over HTTP(S) and from local files
- Caches external stylesheets (opt-in)
- Runs on Linux, Windows and macOS (see [Platform Support](#platform-support))
- Supports HTML5 & CSS3

## Installation

The package is published to [GitHub Packages](https://github.com/Stranger6667/css-inline/packages). It is not on Maven Central yet.

### Setup

GitHub Packages requires authentication, even for public packages. You need a GitHub username and a personal access token (PAT) with the `read:packages` scope. The Gradle snippet reads them from the `gpr.user` / `gpr.key` properties or the `GITHUB_ACTOR` / `GITHUB_TOKEN` environment variables. GitHub Actions sets `GITHUB_ACTOR` on its own. Map `GITHUB_TOKEN` yourself and grant the job read access to packages:

```yaml
permissions:
  packages: read
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
``` See the [GitHub documentation](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-gradle-registry#authenticating-to-github-packages) for details.

**Gradle (Groovy DSL):**
```gradle
repositories {
    maven {
        url = uri("https://maven.pkg.github.com/Stranger6667/css-inline")
        credentials {
            username = project.findProperty("gpr.user") ?: System.getenv("GITHUB_ACTOR")
            password = project.findProperty("gpr.key") ?: System.getenv("GITHUB_TOKEN")
        }
    }
}

dependencies {
    implementation 'org.css-inline:css-inline:0.22.0'
}
```

**Maven:**
```xml
<repositories>
    <repository>
        <id>github</id>
        <url>https://maven.pkg.github.com/Stranger6667/css-inline</url>
    </repository>
</repositories>

<dependencies>
    <dependency>
        <groupId>org.css-inline</groupId>
        <artifactId>css-inline</artifactId>
        <version>0.22.0</version>
    </dependency>
</dependencies>
```

Maven reads the credentials from `~/.m2/settings.xml`. The server `<id>` must match the repository `<id>` above:

```xml
<settings>
    <servers>
        <server>
            <id>github</id>
            <username>YOUR_GITHUB_USERNAME</username>
            <password>YOUR_PAT</password>
        </server>
    </servers>
</settings>
```

See [GitHub's Maven documentation](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-apache-maven-registry).

### Platform Support

The JAR bundles native libraries for:

- **Linux** x86_64
- **macOS** x86_64
- **macOS** aarch64 (Apple Silicon)
- **Windows** x86_64

It requires Java 17+ on a 64-bit JVM. On other platforms, including Linux aarch64 and musl, the first use of `CssInline` throws `UnsatisfiedLinkError`, and later uses throw `NoClassDefFoundError`.

## Usage

```java
import org.cssinline.CssInline;

public class Example {
    public static void main(String[] args) {
        String html = """
            <html>
            <head>
                <style>h1 { color:blue; }</style>
            </head>
            <body>
                <h1>Big Text</h1>
            </body>
            </html>""";

        String inlined = CssInline.inline(html);
        System.out.println(inlined);
    }
}
```

To apply a CSS string to a document, pass it as the second argument. This equals `setExtraCss(css)` on a default config. No overload takes both CSS and a config; use `setExtraCss` for that:

```java
String inlined = CssInline.inline(html, "h1 { color: blue; }");
```

To change the defaults, build a `CssInlineConfig`:

```java
import org.cssinline.CssInline;
import org.cssinline.CssInlineConfig;

public class ConfigExample {
    public static void main(String[] args) {
        String html = "...";

        CssInlineConfig config = new CssInlineConfig.Builder()
            .setLoadRemoteStylesheets(false)
            .setKeepStyleTags(true)
            .setBaseUrl("https://example.com/")
            .build();

        String inlined = CssInline.inline(html, config);
    }
}
```

- **`setInlineStyleTags(boolean)`** - Inline CSS from `<style>` tags (default: `true`)
- **`setKeepStyleTags(boolean)`** - Keep `<style>` tags after inlining (default: `false`)
- **`setKeepLinkTags(boolean)`** - Keep `<link>` tags after inlining (default: `false`)
- **`setKeepAtRules(boolean)`** - Keep `at-rules` (starting with `@`) after inlining (default: `false`)
- **`setMinifyCss(boolean)`** - Remove trailing semicolons and spaces between properties and values (default: `false`)
- **`setBaseUrl(String)`** - Base URL for resolving relative URLs (default: `null`)
- **`setLoadRemoteStylesheets(boolean)`** - Load stylesheets from `link` tags, over the network or from `file://` paths (default: `true`)
- **`setExtraCss(String)`** - Extra CSS to inline (default: `null`)
- **`setCacheSize(int)`** - External stylesheet cache size, `0` disables the cache; the setter throws `IllegalArgumentException` on negative values (default: `0`)
- **`setPreallocateNodeCapacity(int)`** - HTML node capacity; the setter throws `IllegalArgumentException` on values ≤ 0 (default: `32`)
- **`setRemoveInlinedSelectors(boolean)`** - Remove inlined selectors from `<style>` blocks (default: `false`)
- **`setApplyWidthAttributes(boolean)`** - Add `width` HTML attributes from CSS `width` properties on supported elements (`table`, `td`, `th`, `img`) (default: `false`)
- **`setApplyHeightAttributes(boolean)`** - Add `height` HTML attributes from CSS `height` properties on supported elements (`table`, `td`, `th`, `img`) (default: `false`)

`inline` and `inlineFragment` throw `org.cssinline.CssInlineException` (a `RuntimeException`) when inlining fails. A malformed `setBaseUrl` value passes `build()` and fails here, as does a stylesheet that cannot be loaded.

### HTML Fragments

`CssInline.inlineFragment(fragment, css)` applies CSS to an HTML snippet that has no document structure. The `css` argument is required; `inlineFragment(fragment, css, config)` also takes a config. The output contains only the fragment markup: if the input has `<html>`, `<head>` or `<body>` tags, the output removes the tags and keeps their contents. Use `CssInline.inline` to keep the full document:

```java
import org.cssinline.CssInline;

public class FragmentExample {
    public static void main(String[] args) {
        String fragment = """
            <main>
            <h1>Hello</h1>
            <section>
            <p>who am i</p>
            </section>
            </main>""";

        String css = """
            p {
                color: red;
            }

            h1 {
                color: blue;
            }
            """;

        String inlined = CssInline.inlineFragment(fragment, css);
        System.out.println(inlined);
        // <main>
        // <h1 style="color: blue;">Hello</h1>
        // <section>
        // <p style="color: red;">who am i</p>
        // </section>
        // </main>
    }
}
```

### Special Attributes

To skip inlining for a tag, add `data-css-inline="ignore"` to it:

```html
<head>
  <style>h1 { color:blue; }</style>
</head>
<body>
  <!-- The tag below won't receive additional styles -->
  <h1 data-css-inline="ignore">Big Text</h1>
</body>
```

The same attribute on a `link` or `style` tag excludes its CSS:

```html
<head>
  <!-- Styles below are ignored -->
  <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

To keep a `style` tag in the output, add `data-css-inline="keep"`. Responsive emails use this to keep `@media` queries in separate `style` tags.
The tag stays even with `setKeepStyleTags(false)`.

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

At-rules such as `@media` cannot go into a `style` attribute, so the inliner removes them by default. `setKeepAtRules(true)` keeps them in `style` tags and inlines the rest.
The `style` tags that hold these at-rules stay even with `setKeepStyleTags(false)`.

```html
<head>
  <!-- With setKeepAtRules(true), "color:blue" will get inlined into <h1> but @media will be kept in <style> -->
  <style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

`setMinifyCss(true)` removes trailing semicolons and the spaces between properties and values in inlined styles.

```html
<head>
  <!-- With setMinifyCss(true), the <h1> will have `style="color:blue;font-weight:bold"` -->
  <style>h1 { color: blue; font-weight: bold; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

## Performance

`css-inline` against [CSSBox](https://github.com/radkovo/CSSBox), average time per call (lower is better). Bold numbers show how many times slower CSSBox is:

|             | Size    | `css-inline 0.22.0` | `CSSBox 5.0.0`          |
|-------------|---------|---------------------|-------------------------|
| Basic       | 230 B   | 6.95 µs             | 54.13 µs (**7.79x**)    |
| Realistic-1 | 8.58 KB | 100.76 µs           | 1.65 ms (**16.33x**)    |
| Realistic-2 | 4.30 KB | 54.39 µs            | 404.81 µs (**7.44x**)   |
| GitHub page | 1.81 MB | 20.42 ms            | 313.56 ms (**15.35x**)  |

Setup: JMH average-time mode, 1 fork, one 10 s warmup and three 10 s measurement iterations, stable `rustc 1.99`, `OpenJDK 26.0.1`, Ryzen 9 9950X. Inputs come from [`benchmarks/benchmarks.json`](../../benchmarks/benchmarks.json); the code is in [`CSSInlineBench.java`](src/jmh/java/org/cssinline/CSSInlineBench.java).

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).

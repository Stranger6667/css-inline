# css-inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="github packages" src="https://img.shields.io/badge/github%20packages-css--inline-66c2a5?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/packages)

Java bindings for the high-performance `css-inline` library that inlines CSS into HTML 'style' attributes.

This library is designed for scenarios such as preparing HTML emails or embedding HTML into third-party web pages.

Transforms HTML like this:

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

- Uses reliable components from Mozilla's Servo project
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Optionally caches external stylesheets
- Works on Linux, Windows, and macOS
- Supports HTML5 & CSS3

## Installation

This package is available on [GitHub Packages](https://github.com/Stranger6667/css-inline/packages).

### Setup

GitHub Packages requires authentication even for public packages. See the [GitHub documentation](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-gradle-registry#authenticating-to-github-packages) for authentication setup.

**Gradle:**
```gradle
repositories {
    maven {
        url = uri("https://maven.pkg.github.com/Stranger6667/css-inline")
        credentials {
            username = project.findProperty("gpr.user") ?: System.getenv("USERNAME")
            password = project.findProperty("gpr.key") ?: System.getenv("TOKEN")
        }
    }
}

dependencies {
    implementation 'org.css-inline:css-inline:0.15.0'
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
        <version>0.15.0</version>
    </dependency>
</dependencies>
```

See [GitHub's Maven documentation](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-apache-maven-registry) for Maven authentication setup.

### Platform Support

This JAR includes native libraries for the following platforms:

- **Linux** x86_64
- **macOS** x86_64
- **macOS** aarch64 (Apple Silicon)
- **Windows** x86_64

Requires Java 17+ with 64-bit JVM.

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

You can also configure the inlining process:

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
- **`setBaseUrl(String)`** - Base URL for resolving relative URLs (default: `null`)
- **`setLoadRemoteStylesheets(boolean)`** - Load external stylesheets (default: `true`)
- **`setExtraCss(String)`** - Additional CSS to inline (default: `null`)


### HTML Fragments

Alternatively, you can inline CSS into an HTML fragment, preserving the original structure:

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
    }
}
```

The `inlineFragment` method is useful when you have HTML snippets without a full document structure and want to apply specific CSS rules directly.

### Special Attributes

You can also skip CSS inlining for an HTML tag by adding the `data-css-inline="ignore"` attribute to it:

```html
<head>
  <style>h1 { color:blue; }</style>
</head>
<body>
  <!-- The tag below won't receive additional styles -->
  <h1 data-css-inline="ignore">Big Text</h1>
</body>
```

The `data-css-inline="ignore"` attribute also allows you to skip `link` and `style` tags:

```html
<head>
  <!-- Styles below are ignored -->
  <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Alternatively, you may keep `style` from being removed by using the `data-css-inline="keep"` attribute.
This is useful if you want to keep `@media` queries for responsive emails in separate `style` tags:

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Such tags will be kept in the resulting HTML even if the `keepStyleTags` option is set to `false`.

## Performance

`css-inline` is powered by efficient tooling from Mozilla's Servo project to provide high-performance CSS inlining for Java applications.

Here is the performance comparison:

|             | Size    | `css-inline 0.15.0` | `CSSBox 5.0.0`     |
|-------------|---------|---------------------|------------------------|
| Basic       | 230 B   | 7.67 µs             | 209.93 µs (**27.37x**) |
| Realistic-1 | 8.58 KB | 123.18 µs           | 1.92 ms (**15.58x**)   |
| Realistic-2 | 4.3 KB  | 77.74 µs            | 608.65 µs (**7.82x**)   |
| GitHub page | 1.81 MB | 168.43 ms           | 316.21 ms (**1.87x**)  |

The benchmarking code is available in the `src/jmh/java/org/cssinline/CSSInlineBench.java` file. The benchmarks were conducted using the stable `rustc 1.87`, `OpenJDK 24.0.1` on Ryzen 9 9950X.

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).

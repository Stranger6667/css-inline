use assert_cmd::{cargo::cargo_bin_cmd, Command};

fn css_inline() -> Command {
    cargo_bin_cmd!("css-inline")
}

#[cfg(feature = "cli")]
pub mod tests {
    use super::css_inline;
    use std::fs;
    use test_case::test_case;

    #[test]
    fn success() {
        css_inline()
            .arg("tests/example.html")
            .arg("--output-filename-prefix=inlined.success.")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");
        let content = fs::read_to_string("tests/inlined.success.example.html").unwrap();
        assert_eq!(
            content,
            "<html><head>\n    \
        \n    \
        \n    \
        \n\
        </head>\n\
        <body>\n\
        <a class=\"test-class\" href=\"https://example.com\" style=\"color: #ffffff;\">Test</a>\n\
        <h1 style=\"text-decoration: none;\">Test</h1>\n\n\n\
        </body></html>"
        )
    }

    #[test]
    fn keep_style_tags() {
        css_inline()
            .arg("tests/example.html")
            .arg("--keep-style-tags")
            .arg("--output-filename-prefix=inlined.keep-style-tags.")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");
        let content = fs::read_to_string("tests/inlined.keep-style-tags.example.html").unwrap();
        assert_eq!(
            content,
            "<html><head>\n    \n    \
        <style>\n        h1 {\n            text-decoration: none;\n        }\n    </style>\n    \
        <style>\n        .test-class {\n            color: #ffffff;\n        }\n\n        a {\n            color: #17bebb;\n        }\n    </style>\n\
        </head>\n\
        <body>\n\
        <a class=\"test-class\" href=\"https://example.com\" style=\"color: #ffffff;\">Test</a>\n\
        <h1 style=\"text-decoration: none;\">Test</h1>\n\n\n\
        </body></html>"
        )
    }

    #[test]
    fn keep_at_rules() {
        css_inline()
            .arg("tests/example.html")
            .arg("--keep-at-rules")
            .arg("--output-filename-prefix=inlined.keep-at-rules.")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");
        let content = fs::read_to_string("tests/inlined.keep-at-rules.example.html").unwrap();
        assert_eq!(
            content,
            "<html><head>\n    \n    \n    \n\
        </head>\n\
        <body>\n\
        <a class=\"test-class\" href=\"https://example.com\" style=\"color: #ffffff;\">Test</a>\n\
        <h1 style=\"text-decoration: none;\">Test</h1>\n\n\n\
        </body></html>"
        )
    }

    #[test]
    fn minify_css() {
        css_inline()
            .arg("tests/example.html")
            .arg("--minify-css")
            .arg("--output-filename-prefix=inlined.minify-css.")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");
        let content = fs::read_to_string("tests/inlined.minify-css.example.html").unwrap();
        assert_eq!(
            content,
            "<html><head>\n    \n    \n    \n\
        </head>\n\
        <body>\n\
        <a class=\"test-class\" href=\"https://example.com\" style=\"color:#ffffff\">Test</a>\n\
        <h1 style=\"text-decoration:none\">Test</h1>\n\n\n\
        </body></html>"
        )
    }

    #[test]
    fn dont_inline_styles() {
        css_inline()
            .arg("tests/example.html")
            .arg("--inline-style-tags=false")
            .arg("--output-filename-prefix=inlined.dont-inline-styles.")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");
        let content = fs::read_to_string("tests/inlined.dont-inline-styles.example.html").unwrap();
        assert_eq!(
            content,
            "<html><head>\n    \n    \n    \n\
        </head>\n\
        <body>\n\
        <a class=\"test-class\" href=\"https://example.com\">Test</a>\n\
        <h1>Test</h1>\n\n\n\
        </body></html>"
        )
    }

    #[test]
    fn no_remote_stylesheets() {
        css_inline()
            .arg("tests/example.html")
            .arg("--load-remote-stylesheets=false")
            .arg("--output-filename-prefix=inlined.no-remote-stylesheets.")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");
        let content =
            fs::read_to_string("tests/inlined.no-remote-stylesheets.example.html").unwrap();
        assert_eq!(
            content,
            "<html><head>\n    \n    \n    \n\
        </head>\n\
        <body>\n\
        <a class=\"test-class\" href=\"https://example.com\" style=\"color: #ffffff;\">Test</a>\n\
        <h1 style=\"text-decoration: none;\">Test</h1>\n\n\n\
        </body></html>"
        )
    }

    #[test]
    #[cfg(feature = "stylesheet-cache")]
    fn cache_valid() {
        css_inline()
            .arg("tests/example.html")
            .arg("--output-filename-prefix=inlined.cache-valid.")
            .arg("--cache-size=6")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");
    }

    #[test]
    #[cfg(feature = "stylesheet-cache")]
    fn cache_invalid() {
        css_inline()
            .arg("tests/example.html")
            .arg("--output-filename-prefix=inlined.cache-valid.")
            .arg("--cache-size=0")
            .assert()
            .failure()
            .stderr("ERROR: Cache size must be an integer greater than zero\n");
    }

    #[test]
    fn wrong_base_url() {
        css_inline()
            .arg("--base-url=https://:::::")
            .write_stdin(r#"<html><head><link href="external.css" rel="stylesheet"></head><body><h1>Hello world!</h1></body></html>"#)
            .assert()
            .failure()
            .stderr("Status: ERROR\nDetails: empty host\n");
    }

    #[test]
    fn extra_css_file() {
        css_inline()
            .arg("tests/example.html")
            .arg("--extra-css-file=tests/extra.css")
            .arg("--output-filename-prefix=inlined.extra-css.")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");

        let content = fs::read_to_string("tests/inlined.extra-css.example.html").unwrap();
        assert!(
            content.contains(r#"style="color: #ffffff;background: red;""#),
            "inlined output did not include extra-css rules:\n{content}"
        );
    }

    #[test]
    fn extra_css_file_not_found() {
        css_inline()
            .arg("tests/example.html")
            .arg("--extra-css-file=tests/nonexistent.css")
            .assert()
            .failure()
            .stderr(
                "Status: ERROR\n\
                 Details: Failed to read CSS file 'tests/nonexistent.css': No such file or directory (os error 2)\n",
            );
    }

    #[test]
    fn extra_css() {
        css_inline()
            .arg("tests/example.html")
            .arg("--extra-css=.test-class { background: green; }")
            .arg("--output-filename-prefix=inlined.extra-css-cli.")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");

        let content = fs::read_to_string("tests/inlined.extra-css-cli.example.html").unwrap();
        assert!(
            content.contains(r#"style="color: #ffffff;background: green;""#),
            "expected inline background from --extra-css but got:\n{content}",
        );
    }

    #[test]
    fn not_found() {
        css_inline().arg("unknown.html").assert().failure().stderr(
            "Filename: unknown.html\nStatus: ERROR\nDetails: No such file or directory (os error 2)\n",
        );
    }

    #[test]
    fn invalid_css() {
        css_inline()
            .write_stdin(r#"<html><head><style>h1 {background-color: blue;}</style></head><body><h1 style="@wrong { color: ---}">Hello world!</h1></body></html>"#)
            .assert()
            .failure()
            .stderr("Status: ERROR\nDetails: Invalid @ rule: wrong\n");
    }

    #[test]
    fn invalid_css_in_file() {
        css_inline()
            .arg("tests/invalid-example.html")
            .assert()
            .failure()
            .stderr(
                "Filename: tests/invalid-example.html\nStatus: ERROR\nDetails: Invalid @ rule: wrong\n",
            );
    }

    #[test]
    fn stdin() {
        css_inline()
            .write_stdin(r#"<html><head><style>h1 {background-color: blue;}</style></head><body><h1>Hello world!</h1></body></html>"#)
            .assert()
            .success()
            .stdout("<html><head></head><body><h1 style=\"background-color: blue;\">Hello world!</h1></body></html>");
    }

    #[test_case("--help", "css-inline inlines CSS into HTML")]
    #[test_case("-h", "css-inline inlines CSS into HTML")]
    #[test_case("--version", "css-inline")]
    #[test_case("-v", "css-inline")]
    fn args(arg: &str, expected: &str) {
        let stdout = css_inline().arg(arg).assert().success().to_string();
        assert!(stdout.contains(expected), "{}", stdout);
    }

    #[test]
    fn flag_requires_value_but_none_provided() {
        css_inline()
            .arg("--base-url")
            .assert()
            .failure()
            .stderr("Error parsing arguments: Flag --base-url requires a value\n");
    }

    #[test]
    fn invalid_multi_character_short_flag() {
        css_inline()
            .arg("-abc")
            .assert()
            .failure()
            .stderr("Error parsing arguments: Invalid flag: -abc\n");
    }

    #[test]
    fn keep_link_tags_flag() {
        css_inline()
            .arg("tests/example.html")
            .arg("--keep-link-tags")
            .arg("--output-filename-prefix=inlined.keep-link-tags.")
            .assert()
            .success()
            .stdout("tests/example.html: SUCCESS\n");
    }

    #[test]
    fn unknown_short_flag() {
        css_inline()
            .arg("-b")
            .assert()
            .failure()
            .stderr("Unknown flag: b\n");
    }

    #[test]
    fn unknown_boolean_flag() {
        css_inline()
            .arg("--unknown-flag")
            .assert()
            .failure()
            .stderr("Unknown flag: unknown-flag\n");
    }

    #[test]
    fn unknown_flag_with_value() {
        css_inline()
            .arg("--unknown-flag=value")
            .assert()
            .failure()
            .stderr("Unknown flag: --unknown-flag\n");
    }

    #[test]
    fn invalid_boolean_value_for_flag() {
        css_inline()
            .arg("--inline-style-tags=invalid")
            .assert()
            .failure()
            .stderr("Failed to parse value 'invalid' for flag 'inline-style-tags': provided string was not `true` or `false`\n");
    }

    #[test]
    #[cfg(feature = "stylesheet-cache")]
    fn invalid_numeric_value_for_cache_size() {
        css_inline()
        .arg("--cache-size=invalid")
        .assert()
        .failure()
        .stderr("Failed to parse value 'invalid' for flag 'cache-size': invalid digit found in string\n");
    }
}

#[cfg(not(feature = "cli"))]
pub mod tests {
    use super::css_inline;

    #[test]
    fn test_no_cli_feature() {
        let cmd = css_inline().assert().failure();
        let stdout = &cmd.get_output().stderr;
        assert_eq!(
            stdout,
            b"`css-inline` CLI is only available with the `cli` feature\n"
        );
    }
}

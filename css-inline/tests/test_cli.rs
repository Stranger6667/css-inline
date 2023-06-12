use assert_cmd::Command;
use std::fs;
use test_case::test_case;

fn css_inline() -> Command {
    Command::cargo_bin("css-inline").unwrap()
}

#[test]
fn success() {
    css_inline()
        .arg("tests/example.html")
        .arg("--output-filename-prefix=keep-style-tags.")
        .assert()
        .success()
        .stdout("tests/example.html: SUCCESS\n");
    let content = fs::read_to_string("tests/keep-style-tags.example.html").unwrap();
    assert_eq!(
        content,
        "<html><head>\n    \
        <link href=\"external.css\" rel=\"stylesheet\" type=\"text/css\">\n    \
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
        .assert()
        .success()
        .stdout("tests/example.html: SUCCESS\n");
    let content = fs::read_to_string("tests/inlined.example.html").unwrap();
    assert_eq!(
        content,
        "<html><head>\n    \
        <link href=\"external.css\" rel=\"stylesheet\" type=\"text/css\">\n    \
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
fn wrong_base_url() {
    css_inline()
        .arg("--base-url=https://:::::")
        .write_stdin(r#"<html><head><title>Test</title><link href="external.css" rel="stylesheet" type="text/css"></head><body><h1>Hello world!</h1></body></html>"#)
        .assert()
        .failure()
        .stderr("Status: ERROR\nDetails: empty host\n");
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
        .write_stdin(r#"<html><head><title>Test</title><style>h1 {background-color: blue;}</style></head><body><h1 style="@wrong { color: ---}">Hello world!</h1></body></html>"#)
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
        .write_stdin(r#"<html><head><title>Test</title><style>h1 {background-color: blue;}</style></head><body><h1>Hello world!</h1></body></html>"#)
        .assert()
        .success()
        .stdout("<html><head><title>Test</title></head><body><h1 style=\"background-color: blue;\">Hello world!</h1></body></html>");
}

#[test_case("--help", "css-inline inlines CSS into HTML documents.")]
#[test_case("--version", "css-inline 0.9.0")]
fn args(arg: &str, expected: &str) {
    let stdout = css_inline().arg(arg).assert().success().to_string();
    assert!(stdout.contains(expected), "{}", stdout);
}

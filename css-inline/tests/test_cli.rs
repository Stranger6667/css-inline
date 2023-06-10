use assert_cmd::Command;
use std::fs;

#[test]
fn success() {
    let mut cmd = Command::cargo_bin("css-inline").unwrap();
    cmd.arg("tests/example.html")
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
fn remove_style_tags() {
    let mut cmd = Command::cargo_bin("css-inline").unwrap();
    cmd.arg("tests/example.html")
        .arg("--output-filename-prefix=remove-style-tags.")
        .assert()
        .success()
        .stdout("tests/example.html: SUCCESS\n");
    let content = fs::read_to_string("tests/remove-style-tags.example.html").unwrap();
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
fn not_found() {
    let mut cmd = Command::cargo_bin("css-inline").unwrap();
    let assert = cmd.arg("unknown.html").assert();
    assert
        .failure()
        .stdout("unknown.html: FAILURE (No such file or directory (os error 2))\n");
}

use css_inline::inline;

const HTML: &str = r#"<html>
<head>
<title>Test</title>
<style>
h1, h2 { color:red; }
strong {
  text-decoration:none
  }
p { font-size:2px }
p.footer { font-size: 1px}
</style>
</head>
<body>
<h1>Hi!</h1>
<p><strong>Yes!</strong></p>
<p class="footer">Feetnuts</p>
</body>
</html>"#;

fn main() {
    for _ in 0..1000 {
        let _ = inline(HTML);
    }
}

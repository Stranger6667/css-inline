#![cfg(feature = "async")]
use css_inline::{CSSInliner, StylesheetResolver, Url};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

const HTML: &str = r#"<html>
<head>
<link href="a.css" rel="stylesheet">
<link href="b.css" rel="stylesheet">
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;

fn stylesheet(location: &str) -> css_inline::Result<String> {
    match location.rsplit('/').next() {
        Some("a.css") => Ok("h1 { color: blue; }".to_string()),
        Some("b.css") => Ok("h2 { color: red; }".to_string()),
        _ => Err(std::io::Error::new(std::io::ErrorKind::NotFound, location.to_string()).into()),
    }
}

#[derive(Debug, Default)]
struct Counting {
    hits: AtomicUsize,
    in_flight: AtomicUsize,
    max_in_flight: AtomicUsize,
}

#[css_inline::async_trait]
impl StylesheetResolver for Counting {
    async fn retrieve_async(&self, location: &str) -> css_inline::Result<String> {
        self.hits.fetch_add(1, Ordering::SeqCst);
        let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_in_flight.fetch_max(now, Ordering::SeqCst);
        tokio::task::yield_now().await;
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        stylesheet(location)
    }
}

struct Blocking;

impl StylesheetResolver for Blocking {
    fn retrieve(&self, location: &str) -> css_inline::Result<String> {
        stylesheet(location)
    }
}

#[tokio::test]
async fn fetches_stylesheets_concurrently() {
    let resolver = Arc::new(Counting::default());
    let inliner = CSSInliner::options().resolver(resolver.clone()).build();
    inliner.inline_async(HTML).await.expect("Inlining failed");
    assert_eq!(resolver.hits.load(Ordering::SeqCst), 2);
    assert_eq!(resolver.max_in_flight.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn matches_blocking_output() {
    let expected = CSSInliner::options()
        .resolver(Arc::new(Blocking))
        .build()
        .inline(HTML)
        .expect("Inlining failed");
    let inliner = CSSInliner::options()
        .resolver(Arc::new(Counting::default()))
        .build();
    assert_eq!(
        inliner.inline_async(HTML).await.expect("Inlining failed"),
        expected
    );
    let mut out = Vec::new();
    inliner
        .inline_to_async(HTML, &mut out)
        .await
        .expect("Inlining failed");
    assert_eq!(String::from_utf8(out).expect("Invalid UTF-8"), expected);
}

#[tokio::test]
async fn fragment_matches_blocking_output() {
    let fragment = r#"<link href="a.css" rel="stylesheet"><h1>Big Text</h1><h2>Smaller Text</h2>"#;
    let css = "h2 { font-size: 1px; }";
    let expected = CSSInliner::options()
        .resolver(Arc::new(Blocking))
        .build()
        .inline_fragment(fragment, css)
        .expect("Inlining failed");
    let inliner = CSSInliner::options()
        .resolver(Arc::new(Counting::default()))
        .build();
    assert_eq!(
        inliner
            .inline_fragment_async(fragment, css)
            .await
            .expect("Inlining failed"),
        expected
    );
}

#[tokio::test]
async fn falls_back_to_blocking_retrieve() {
    let expected = CSSInliner::options()
        .resolver(Arc::new(Blocking))
        .build()
        .inline(HTML)
        .expect("Inlining failed");
    let inlined = CSSInliner::options()
        .resolver(Arc::new(Blocking))
        .build()
        .inline_async(HTML)
        .await
        .expect("Inlining failed");
    assert_eq!(inlined, expected);
}

#[tokio::test]
async fn fetches_each_full_url_once() {
    let html = r#"<link href="a.css" rel="stylesheet"><link href="./a.css" rel="stylesheet"><h1>Big Text</h1>"#;
    let resolver = Arc::new(Counting::default());
    let inliner = CSSInliner::options()
        .base_url(Some(
            Url::parse("http://127.0.0.1:1234/").expect("Invalid URL"),
        ))
        .resolver(resolver.clone())
        .build();
    let inlined = inliner.inline_async(html).await.expect("Inlining failed");
    assert!(inlined.contains(r#"<h1 style="color: blue;">"#));
    assert_eq!(resolver.hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn skips_fetching_when_remote_stylesheets_are_disabled() {
    let resolver = Arc::new(Counting::default());
    let inliner = CSSInliner::options()
        .load_remote_stylesheets(false)
        .resolver(resolver.clone())
        .build();
    inliner.inline_async(HTML).await.expect("Inlining failed");
    assert_eq!(resolver.hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn propagates_resolver_errors() {
    let html = r#"<link href="missing.css" rel="stylesheet"><h1>Big Text</h1>"#;
    let inliner = CSSInliner::options()
        .resolver(Arc::new(Counting::default()))
        .build();
    let error = inliner.inline_async(html).await.expect_err("Should fail");
    assert_eq!(error.to_string(), "missing.css");
}

#[tokio::test]
#[cfg(feature = "stylesheet-cache")]
async fn does_not_fetch_cached_stylesheets() {
    let resolver = Arc::new(Counting::default());
    let inliner = CSSInliner::options()
        .resolver(resolver.clone())
        .cache(css_inline::StylesheetCache::new(
            std::num::NonZeroUsize::new(3).expect("Invalid size"),
        ))
        .build();
    for _ in 0..3 {
        inliner.inline_async(HTML).await.expect("Inlining failed");
    }
    assert_eq!(resolver.hits.load(Ordering::SeqCst), 2);
}

#[test]
fn inlining_future_is_send() {
    fn assert_send<T: Send>(_: T) {}
    let inliner = CSSInliner::default();
    assert_send(inliner.inline_async(HTML));
}

#[tokio::test]
#[cfg(feature = "http")]
async fn default_resolver_fetches_over_http() {
    let html =
        r#"<link href="http://127.0.0.1:1234/external.css" rel="stylesheet"><h1>Big Text</h1>"#;
    let inlined = CSSInliner::default()
        .inline_async(html)
        .await
        .expect("Inlining failed");
    assert!(inlined.contains(r#"<h1 style="color: blue;">"#));
}

#[tokio::test]
async fn shortcuts_match_blocking_output() {
    let html = "<html><head><style>h1 { color: blue; }</style></head><body><h1>Big Text</h1></body></html>";
    let fragment = "<h1>Big Text</h1>";
    let css = "h1 { color: blue; }";
    assert_eq!(
        css_inline::inline_async(html)
            .await
            .expect("Inlining failed"),
        css_inline::inline(html).expect("Inlining failed")
    );
    let mut out = Vec::new();
    css_inline::inline_to_async(html, &mut out)
        .await
        .expect("Inlining failed");
    assert_eq!(
        String::from_utf8(out).expect("Invalid UTF-8"),
        css_inline::inline(html).expect("Inlining failed")
    );
    assert_eq!(
        css_inline::inline_fragment_async(fragment, css)
            .await
            .expect("Inlining failed"),
        css_inline::inline_fragment(fragment, css).expect("Inlining failed")
    );
    let mut out = Vec::new();
    css_inline::inline_fragment_to_async(fragment, css, &mut out)
        .await
        .expect("Inlining failed");
    assert_eq!(
        String::from_utf8(out).expect("Invalid UTF-8"),
        css_inline::inline_fragment(fragment, css).expect("Inlining failed")
    );
}

#[tokio::test]
async fn default_resolver_reports_invalid_url() {
    let html = r#"<link href="http:" rel="stylesheet"><h1>Big Text</h1>"#;
    let error = CSSInliner::default()
        .inline_async(html)
        .await
        .expect_err("Should fail");
    #[cfg(feature = "http")]
    let expected = "builder error: http:";
    #[cfg(not(feature = "http"))]
    let expected = "Loading external URLs requires the `http` feature";
    assert_eq!(error.to_string(), expected);
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
#[cfg(all(unix, feature = "file"))]
fn default_resolver_reads_files_without_blocking_executor() {
    // Opening a FIFO blocks until a writer opens it, and the writer runs on the same executor thread
    let dir = std::env::temp_dir().join(format!("css-inline-fifo-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("Failed to create a directory");
    let fifo = dir.join("style.css");
    let _ = std::fs::remove_file(&fifo);
    assert!(std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("Failed to run mkfifo")
        .success());
    let html = format!(
        r#"<link href="{}" rel="stylesheet"><h1>Big Text</h1>"#,
        fifo.display()
    );
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("Failed to build a runtime");
        let inliner = CSSInliner::default();
        let (inlined, ()) = runtime.block_on(async {
            tokio::join!(inliner.inline_async(&html), async {
                std::fs::write(&fifo, "h1 { color: blue; }").expect("Failed to write");
            })
        });
        tx.send(inlined).expect("Failed to send");
    });
    let inlined = rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("Reading a local stylesheet blocked the executor")
        .expect("Inlining failed");
    assert!(inlined.contains(r#"<h1 style="color: blue;">"#));
}

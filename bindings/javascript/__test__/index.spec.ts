import test from "ava";

import { inline } from "../index.js";

test("default inlining", (t) => {
  t.is(
    inline(
      "<html><head><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
    ),
    '<html><head></head><body><h1 style="color: red;">Test</h1></body></html>',
  );
});

test("keep style tag", (t) => {
  t.is(
    inline(
      "<html><head><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
      { keepStyleTags: true },
    ),
    '<html><head><style>h1 { color:red; }</style></head><body><h1 style="color: red;">Test</h1></body></html>',
  );
});

test("valid baseURL", (t) => {
  t.is(
    inline(
      "<html><head><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
      { baseUrl: "http://127.0.0.1" },
    ),
    '<html><head></head><body><h1 style="color: red;">Test</h1></body></html>',
  );
});

test("invalid baseURL", (t) => {
  const error = t.throws(() => {
    inline(
      "<html><head><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
      { baseUrl: "foo" },
    );
  });
  t.is(error.code, "InvalidArg");
  t.is(error.message, "relative URL without a base: foo");
});

test("invalid href", (t) => {
  const error = t.throws(() => {
    inline(
      "<html><head><link href='http:' rel='stylesheet' type='text/css'></head><body></body></html>",
    );
  });
  t.is(error.code, "GenericFailure");
  t.is(error.message, "builder error: empty host: http:");
});

test("invalid style", (t) => {
  const error = t.throws(() => {
    inline(
      "<html><head><style>h1, h2 { color:red; }</style></head><body><h1 style='@wrong { color: --- }'>Hello world!</h1></body></html>",
    );
  });
  t.is(error.code, "GenericFailure");
  t.is(error.message, "Invalid @ rule: wrong");
});

const inlinedHtml = `<html><head>

<link href="/rss.xml" rel="alternate" title="RSS" type="application/rss+xml">

</head>
<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>`;

test("remote file stylesheet", (t) => {
  t.is(
    inline(`<html>
<head>
<link href="../../css-inline/tests/external.css" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>`),
    inlinedHtml,
  );
});

test("remote network stylesheet", (t) => {
  t.is(
    inline(`<html>
<head>
<link href="http://127.0.0.1:1234/external.css" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>`),
    inlinedHtml,
  );
});

test("remote network relative stylesheet", (t) => {
  t.is(
    inline(
      `<html>
<head>
<link href="external.css" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>`,
      { baseUrl: "http://127.0.0.1:1234" },
    ),
    inlinedHtml,
  );
});

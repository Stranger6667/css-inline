import { createServer } from "http";
import { AddressInfo } from "net";

import test from "ava";

import {
  inline,
  inlineAsync,
  inlineFragment,
  inlineFragmentAsync,
} from "../index.js";

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
  t.is(error.message, "builder error: http:");
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

<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">

</head>
<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>`;

test("remote file stylesheet", (t) => {
  t.is(
    inline(`<html>
<head>
<link href="../css-inline/tests/external.css" rel="stylesheet">
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

test("cache external stylesheets", (t) => {
  t.is(
    inline(
      `<html>
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
</html>`,
      { cache: { size: 5 } },
    ),
    inlinedHtml,
  );
});

test("invalid cache size", (t) => {
  const error = t.throws(() => {
    inline("", { cache: { size: 0 } });
  });
  t.is(error.code, "GenericFailure");
  t.is(error.message, "Cache size must be an integer greater than zero");
});

test("inline fragment", (t) => {
  t.is(
    inlineFragment(
      `<main>
<h1>Hello</h1>
<section>
<p>who am i</p>
</section>
</main>
`,
      `p {
    color: red;
}

h1 {
    color: blue;
}`,
    ),
    `<main>\n<h1 style="color: blue;">Hello</h1>\n<section>\n<p style="color: red;">who am i</p>\n</section>\n</main>\n`,
  );
});

test("remove inlined selectors", (t) => {
  t.is(
    inline(
      "<html><head><style>h1 { color: blue; } h2 { color: red; }</style></head><body><h1>Test</h1></body></html>",
      { removeInlinedSelectors: true },
    ),
    '<html><head><style>h2 { color: red; }</style></head><body><h1 style="color: blue;">Test</h1></body></html>',
  );
});

const remoteHtml = `<html>
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
</html>`;

test("async remote network stylesheet", async (t) => {
  t.is(await inlineAsync(remoteHtml), inlinedHtml);
});

test("async remote network relative stylesheet", async (t) => {
  t.is(
    await inlineAsync(remoteHtml.replace("http://127.0.0.1:1234/", ""), {
      baseUrl: "http://127.0.0.1:1234",
    }),
    inlinedHtml,
  );
});

test("async remote file stylesheet", async (t) => {
  t.is(
    await inlineAsync(
      remoteHtml.replace(
        "http://127.0.0.1:1234/external.css",
        "../css-inline/tests/external.css",
      ),
    ),
    inlinedHtml,
  );
});

test("async matches sync with cache", async (t) => {
  const options = { cache: { size: 2 } };
  t.is(await inlineAsync(remoteHtml, options), inline(remoteHtml, options));
  t.is(await inlineAsync(remoteHtml, options), inlinedHtml);
});

test("async inline fragment", async (t) => {
  t.is(
    await inlineFragmentAsync(
      '<link href="http://127.0.0.1:1234/external.css" rel="stylesheet"><h1>Test</h1><h2>Sub</h2>',
      "h2 { color: red; }",
    ),
    '<h1 style="color: blue;">Test</h1><h2 style="color: red;">Sub</h2>',
  );
});

test("async network error", async (t) => {
  const error = await t.throwsAsync(
    inlineAsync(
      '<html><head><link href="http://127.0.0.1:1/external.css" rel="stylesheet"></head><body></body></html>',
    ),
  );
  t.is(error.code, "GenericFailure");
  t.true(error.message.endsWith(": http://127.0.0.1:1/external.css"));
});

test("async invalid base URL", async (t) => {
  const error = await t.throwsAsync(inlineAsync("", { baseUrl: "foo" }));
  t.is(error.code, "InvalidArg");
  t.is(error.message, "relative URL without a base: foo");
});

test("async does not block the event loop", async (t) => {
  // The stylesheet is served from this event loop, so a blocking fetch would never get a response
  const server = createServer((_, response) =>
    response.end("h1 { color: blue; }"),
  );
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address() as AddressInfo;
  try {
    t.is(
      await inlineAsync(
        `<html><head><link href="http://127.0.0.1:${port}/style.css" rel="stylesheet"></head><body><h1>Test</h1></body></html>`,
      ),
      '<html><head></head><body><h1 style="color: blue;">Test</h1></body></html>',
    );
  } finally {
    server.close();
  }
});

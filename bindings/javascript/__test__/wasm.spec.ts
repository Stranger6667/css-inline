import { promises as fs } from "fs";
import { join } from "path";

import test from "ava";

import { inline, inlineFragment, initWasm } from "../wasm";

test.before(async () => {
  await initWasm(fs.readFile(join(__dirname, "../wasm/index_bg.wasm")));
});

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
  const error = t.throws(
    () => {
      inline(
        "<html><head><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
        { baseUrl: "foo" },
      );
    },
    { any: true },
  );
  t.is(error, "relative URL without a base: foo");
});

test("invalid href", (t) => {
  const error = t.throws(
    () => {
      inline(
        "<html><head><link href='http:' rel='stylesheet' type='text/css'></head><body></body></html>",
      );
    },
    { any: true },
  );
  t.is(error, "Loading remote stylesheets is not supported on WASM: http:");
});

test("invalid style", (t) => {
  const error = t.throws(
    () => {
      inline(
        "<html><head><style>h1, h2 { color:red; }</style></head><body><h1 style='@wrong { color: --- }'>Hello world!</h1></body></html>",
      );
    },
    { any: true },
  );
  t.is(error, "Invalid @ rule: wrong");
});

test("unsupported network operation", (t) => {
  const error = t.throws(
    () => {
      inline(
        "<html><head><link href='http://127.0.0.1:1234/external.css' rel='stylesheet' type='text/css'></head><body></body></html>",
      );
    },
    { any: true },
  );
  t.is(
    error,
    "Loading remote stylesheets is not supported on WASM: http://127.0.0.1:1234/external.css",
  );
});

test("unsupported filesystem operation", (t) => {
  const error = t.throws(
    () => {
      inline(
        "<html><head><link href='tests/external.css' rel='stylesheet' type='text/css'></head><body></body></html>",
      );
    },
    { any: true },
  );
  t.is(
    error,
    "Loading local files is not supported on WASM: tests/external.css",
  );
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
    `<main>\n<h1 style="color: blue;">Hello</h1>\n<section>\n<p style="color: red;">who am i</p>\n</section>\n</main>`,
  );
});

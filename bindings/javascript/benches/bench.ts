import { promises as fs } from "fs";
import { join } from "path";

import b from "benny";
import inlineCss from "inline-css";
import juice from "juice";

import { inline } from "../index";
import { initWasm, inline as wasmInline } from "../wasm";

async function run() {
  const benchmarksPath = join(__dirname, "../../../benchmarks/benchmarks.json");
  const benches = JSON.parse(await fs.readFile(benchmarksPath, "utf-8"));
  await initWasm(fs.readFile(join(__dirname, "../wasm/index_bg.wasm")));

  for (const { name, html } of benches) {
    await b.suite(
      name,
      b.add("css-inline", () => {
        inline(html);
      }),
      b.add("css-inline-wasm", () => {
        wasmInline(html);
      }),
      b.add("juice", () => {
        juice(html);
      }),
      b.add("inline-css", () => {
        inlineCss(html);
      }),
      b.cycle(),
      b.complete(),
    );
  }
}

run().catch((e) => {
  console.error(e);
});

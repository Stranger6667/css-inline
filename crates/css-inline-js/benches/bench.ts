import { promises as fs } from "fs";
import { join } from "path";

import b from "benny";
import inlineCss from "inline-css";
import juice from "juice";

import { inline } from "../index";
import { initWasm, inline as wasmInline } from "../wasm";

// `inline-css` throws parse errors from a callback, so they never reach the promise
function works(fn: () => unknown): Promise<unknown> {
  return new Promise((resolve) => {
    const onError = (e: unknown) => resolve(e);
    process.once("uncaughtException", onError);
    Promise.resolve()
      .then(fn)
      .then(
        () => resolve(null),
        (e) => resolve(e),
      )
      .finally(() => process.off("uncaughtException", onError));
  });
}

async function run() {
  const benchmarksPath = join(__dirname, "../../../benchmarks/benchmarks.json");
  const benches = JSON.parse(await fs.readFile(benchmarksPath, "utf-8"));
  await initWasm(fs.readFile(join(__dirname, "../wasm/index_bg.wasm")));

  for (const { name, html } of benches) {
    // Use more samples for big_page benchmark to get better resolution
    const options = name === "big_page" ? { minSamples: 20 } : {};
    const libraries: [string, () => unknown][] = [
      ["css-inline", () => inline(html)],
      ["css-inline-wasm", () => wasmInline(html)],
      ["juice", () => juice(html)],
      ["inline-css", () => inlineCss(html, { url: "/" })],
    ];
    const cases = [];
    for (const [library, fn] of libraries) {
      const error = await works(fn);
      if (error) {
        // eslint-disable-next-line no-console
        console.log(`Skipping ${library} on ${name}: ${error}`);
        continue;
      }
      // Only `inline-css` is async; deferred benchmarks add overhead to the sync ones
      const bench =
        library === "inline-css"
          ? async () => {
              await fn();
            }
          : fn;
      cases.push(b.add(library, bench, options));
    }
    await b.suite(
      name,
      ...cases,
      b.cycle(),
      b.complete((summary) => {
        // For slow benchmarks, print precise timing from mean values
        const fastest = summary.results[0];
        if (fastest.ops < 100) {
          // eslint-disable-next-line no-console
          console.log("\nPrecise timing (mean):");
          const fastestMean = fastest.details.mean;
          for (const result of summary.results) {
            const mean = result.details.mean;
            const timeStr =
              mean >= 1
                ? `${mean.toFixed(2)} s`
                : `${(mean * 1000).toFixed(2)} ms`;
            const ratio =
              result === fastest
                ? ""
                : ` (${(mean / fastestMean).toFixed(2)}x slower)`;
            // eslint-disable-next-line no-console
            console.log(`  ${result.name}: ${timeStr}${ratio}`);
          }
        }
      }),
    );
  }
}

run().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});

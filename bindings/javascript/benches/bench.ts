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
    // Use more samples for big_page benchmark to get better resolution
    const options = name === "big_page" ? { minSamples: 20 } : {};
    await b.suite(
      name,
      b.add(
        "css-inline",
        () => {
          inline(html);
        },
        options,
      ),
      b.add(
        "css-inline-wasm",
        () => {
          wasmInline(html);
        },
        options,
      ),
      b.add(
        "juice",
        () => {
          juice(html);
        },
        options,
      ),
      b.add(
        "inline-css",
        () => {
          inlineCss(html);
        },
        options,
      ),
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
});

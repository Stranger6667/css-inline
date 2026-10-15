import init, {
  InitInput,
  InlineOptions,
  inline as __inline,
  inlineFragment as __inlineFragment,
  version as __version,
} from "./wasm/dist";

let initialized = false;

/**
 * Initialize Wasm module
 * @param module_or_path WebAssembly Module or .wasm url
 *
 */
export const initWasm = async (
  module_or_path: Promise<InitInput> | InitInput,
): Promise<void> => {
  if (initialized) {
    throw new Error(
      "Already initialized. The `initWasm()` function can be used only once.",
    );
  }
  await init(await module_or_path);
  initialized = true;
};

export function inline(html: string, options?: InlineOptions): string {
  return __inline(html, options);
}

export function inlineFragment(
  html: string,
  css: string,
  options?: InlineOptions,
): string {
  return __inlineFragment(html, css, options);
}

export function version(): string {
  return __version();
}

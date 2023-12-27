/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface Options {
  /**
   * Whether to inline CSS from "style" tags.
   *
   * Sometimes HTML may include a lot of boilerplate styles, that are not applicable in every
   * scenario, and it is useful to ignore them and use `extra_css` instead.
   */
  inlineStyleTags?: boolean
  /** Keep "style" tags after inlining. */
  keepStyleTags?: boolean
  /** Keep "link" tags after inlining. */
  keepLinkTags?: boolean
  /** Used for loading external stylesheets via relative URLs. */
  baseUrl?: string
  /** Whether remote stylesheets should be loaded or not. */
  loadRemoteStylesheets?: boolean
  /** Additional CSS to inline. */
  extraCss?: string
  /**
   * Pre-allocate capacity for HTML nodes during parsing.
   * It can improve performance when you have an estimate of the number of nodes in your HTML document.
   */
  preallocateNodeCapacity?: number
}
/** Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string. */
export function inline(html: string, options?: Options | undefined | null): string
/** Get the package version. */
export function version(): string

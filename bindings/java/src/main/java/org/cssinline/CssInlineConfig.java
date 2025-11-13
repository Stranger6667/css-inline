package org.cssinline;

/** Configuration options for CSS inlining. */
public class CssInlineConfig {
	/** Whether to inline CSS from "style" tags. */
	public final boolean inlineStyleTags;

	/** Keep "style" tags after inlining. */
	public final boolean keepStyleTags;

	/** Keep "link" tags after inlining. */
	public final boolean keepLinkTags;

	/** Keep "at-rules" after inlining. */
	public final boolean keepAtRules;

	/** Remove trailing semicolons and spaces between properties and values. */
	public final boolean minifyCss;

	/** Whether remote stylesheets should be loaded or not. */
	public final boolean loadRemoteStylesheets;

	/** Used for loading external stylesheets via relative URLs. */
	public final String baseUrl;

	/** Additional CSS to inline. */
	public final String extraCss;

	/** External stylesheet cache size. */
	public final int cacheSize;

	/** Pre-allocate capacity for HTML nodes during parsing. */
	public final int preallocateNodeCapacity;

	/** Remove selectors that were successfully inlined from inline style blocks. */
	public final boolean removeInlinedSelectors;

	private CssInlineConfig(boolean inlineStyleTags, boolean keepStyleTags, boolean keepLinkTags,
			boolean keepAtRules, boolean minifyCss, boolean loadRemoteStylesheets, String baseUrl, String extraCss,
			int cacheSize, int preallocateNodeCapacity, boolean removeInlinedSelectors) {
		this.inlineStyleTags = inlineStyleTags;
		this.keepStyleTags = keepStyleTags;
		this.keepLinkTags = keepLinkTags;
		this.keepAtRules = keepAtRules;
		this.minifyCss = minifyCss;
		this.loadRemoteStylesheets = loadRemoteStylesheets;
		this.baseUrl = baseUrl;
		this.extraCss = extraCss;
		this.cacheSize = cacheSize;
		this.preallocateNodeCapacity = preallocateNodeCapacity;
		this.removeInlinedSelectors = removeInlinedSelectors;
	}

	/**
	 * Builder for creating {@link CssInlineConfig} instances.
	 */
	public static class Builder {
		private boolean inlineStyleTags = true;
		private boolean keepStyleTags = false;
		private boolean keepLinkTags = false;
		private boolean keepAtRules = false;
		private boolean minifyCss = false;
		private boolean loadRemoteStylesheets = true;
		private String baseUrl = null;
		private String extraCss = null;
		private int cacheSize = 0;
		private int preallocateNodeCapacity = 32;
		private boolean removeInlinedSelectors = false;

		/**
		 * Creates a new builder with default configuration values.
		 */
		public Builder() {
			// Default constructor for builder
		}

		/**
		 * Whether to inline CSS from "style" tags.
		 * Sometimes HTML may include boilerplate styles that are not applicable
		 * in every scenario and it is useful to ignore them and use extraCss instead.
		 *
		 * @param b true to inline CSS from style tags, false to ignore them
		 * @return this builder instance for method chaining
		 */
		public Builder setInlineStyleTags(boolean b) {
			this.inlineStyleTags = b;
			return this;
		}

		/**
		 * Keep "style" tags after inlining.
		 *
		 * @param b true to preserve style tags, false to remove them
		 * @return this builder instance for method chaining
		 */
		public Builder setKeepStyleTags(boolean b) {
			this.keepStyleTags = b;
			return this;
		}

		/**
		 * Keep "link" tags after inlining.
		 *
		 * @param b true to preserve link tags, false to remove them
		 * @return this builder instance for method chaining
		 */
		public Builder setKeepLinkTags(boolean b) {
			this.keepLinkTags = b;
			return this;
		}

		/**
		 * Keep "at-rules" after inlining.
		 *
		 * @param b true to preserve at-rules, false to remove them
		 * @return this builder instance for method chaining
		 */
		public Builder setKeepAtRules(boolean b) {
			this.keepAtRules = b;
			return this;
		}

		/**
		 * Remove trailing semicolons and spaces between properties and values.
		 *
		 * @param b true to remove, false to keep them
		 * @return this builder instance for method chaining
		 */
		public Builder setMinifyCss(boolean b) {
			this.minifyCss = b;
			return this;
		}

		/**
		 * Whether remote stylesheets should be loaded or not.
		 *
		 * @param b true to load external stylesheets, false to ignore them
		 * @return this builder instance for method chaining
		 */
		public Builder setLoadRemoteStylesheets(boolean b) {
			this.loadRemoteStylesheets = b;
			return this;
		}

		/**
		 * Used for loading external stylesheets via relative URLs.
		 *
		 * @param url the base URL as a string, or null to use no base URL
		 * @return this builder instance for method chaining
		 */
		public Builder setBaseUrl(String url) {
			this.baseUrl = url;
			return this;
		}

		/**
		 * Additional CSS to inline.
		 *
		 * @param css additional CSS rules as a string, or null for no extra CSS
		 * @return this builder instance for method chaining
		 */
		public Builder setExtraCss(String css) {
			this.extraCss = css;
			return this;
		}

		/**
		 * External stylesheet cache size.
		 *
		 * @param size
		 *            cache size, must be non-negative
		 * @return this builder instance for method chaining
		 * @throws IllegalArgumentException
		 *             if size is negative
		 */
		public Builder setCacheSize(int size) {
			if (size < 0) {
				throw new IllegalArgumentException("Cache size must be non-negative, got: " + size);
			}
			this.cacheSize = size;
			return this;
		}

		/**
		 * Pre-allocate capacity for HTML nodes during parsing. Can improve performance
		 * when you have an estimate of the number of nodes in your HTML document.
		 *
		 * @param cap
		 *            initial node capacity, must be positive
		 * @return this builder instance for method chaining
		 * @throws IllegalArgumentException
		 *             if cap is zero or negative
		 */
		public Builder setPreallocateNodeCapacity(int cap) {
			if (cap <= 0) {
				throw new IllegalArgumentException("Preallocate node capacity must be positive, got: " + cap);
			}
			this.preallocateNodeCapacity = cap;
			return this;
		}

		/**
		 * Remove selectors that were successfully inlined from inline style blocks.
		 *
		 * @param b true to remove inlined selectors, false to keep them
		 * @return this builder instance for method chaining
		 */
		public Builder setRemoveInlinedSelectors(boolean b) {
			this.removeInlinedSelectors = b;
			return this;
		}

		/**
		 * Creates a new {@link CssInlineConfig} instance with the current builder settings.
		 *
		 * @return a new immutable configuration instance
		 */
		public CssInlineConfig build() {
			return new CssInlineConfig(inlineStyleTags, keepStyleTags, keepLinkTags, keepAtRules, minifyCss, loadRemoteStylesheets, baseUrl,
					extraCss, cacheSize, preallocateNodeCapacity, removeInlinedSelectors);
		}
	}
}

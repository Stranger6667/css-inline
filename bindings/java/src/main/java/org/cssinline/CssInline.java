package org.cssinline;

/**
 * Main entry point for CSS inlining.
 *
 * Inlines CSS styles from &lt;style&gt; and &lt;link&gt; tags directly into
 * HTML element style attributes. Useful for preparing HTML emails or embedding
 * HTML content where external stylesheets are not supported.
 */
public class CssInline {
	static {
		NativeLibraryLoader.loadLibrary();
	}

	/**
	 * Private constructor to prevent instantiation of utility class.
	 */
	private CssInline() {}

	private static native String nativeInline(String html, CssInlineConfig cfg);
	private static native String nativeInlineFragment(String html, String css, CssInlineConfig cfg);

	/**
	 * Inlines CSS styles into HTML elements using the specified configuration.
	 *
	 * @param html
	 *            the HTML document to process
	 * @param cfg
	 *            the configuration object specifying inlining behavior
	 * @return the HTML document with CSS styles inlined
	 * @throws CssInlineException
	 *             if an error occurs during processing
	 */
	public static String inline(String html, CssInlineConfig cfg) {
		return nativeInline(html, cfg);
	}

	/**
	 * Inlines CSS styles into HTML elements using default configuration.
	 *
	 * @param html
	 *            the HTML document to process
	 * @return the HTML document with CSS styles inlined
	 * @throws CssInlineException
	 *             if an error occurs during processing
	 */
	public static String inline(String html) {
		return inline(html, new CssInlineConfig.Builder().build());
	}

	/**
	 * Inlines the provided CSS into an HTML fragment using the specified configuration.
	 *
	 * <p>Unlike {@link #inline(String, CssInlineConfig)}, this method works with HTML fragments
	 * (elements without &lt;html&gt;, &lt;head&gt;, or &lt;body&gt; tags) and applies the
	 * provided CSS directly without parsing &lt;style&gt; or &lt;link&gt; tags.
	 *
	 * @param fragment the HTML fragment to process
	 * @param css the CSS rules to inline
	 * @param cfg the configuration object specifying inlining behavior
	 * @return the HTML fragment with CSS styles inlined
	 * @throws CssInlineException if an error occurs during processing
	 */
	public static String inlineFragment(String fragment, String css, CssInlineConfig cfg) {
		return nativeInlineFragment(fragment, css, cfg);
	}

	/**
	 * Inlines the provided CSS into an HTML fragment using default configuration.
	 *
	 * <p>Unlike {@link #inline(String)}, this method works with HTML fragments
	 * (elements without &lt;html&gt;, &lt;head&gt;, or &lt;body&gt; tags) and applies the
	 * provided CSS directly without parsing &lt;style&gt; or &lt;link&gt; tags.
	 *
	 * @param fragment the HTML fragment to process
	 * @param css the CSS rules to inline
	 * @return the HTML fragment with CSS styles inlined
	 * @throws CssInlineException if an error occurs during processing
	 */
	public static String inlineFragment(String fragment, String css) {
		return inlineFragment(fragment, css, new CssInlineConfig.Builder().build());
	}
}

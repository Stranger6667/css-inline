package org.cssinline;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;

class CssInlineTest {

	@Test
	void inlinesSimpleStyleTag() {
		String html = "<html><head><style>h1 { color: blue; }</style></head><body><h1>Hello</h1></body></html>";

		String out = CssInline.inline(html);

		assertTrue(out.contains("style=\"color: blue;\""), "Output should inline styles for h1, got: " + out);
	}

	@Test
	void extraCssAddsBackground() {
		String html = "<html><head></head><body><h1>Hello</h1></body></html>";

		CssInlineConfig cfg = new CssInlineConfig.Builder().setExtraCss("h1 { color: blue; }").build();

		String out = CssInline.inline(html, cfg);

		assertTrue(out.contains("style=\"color: blue;\""), "Output should inline styles for h1, got: " + out);
	}

	@Test
	void validBaseUrlParses() {
		CssInlineConfig cfg = new CssInlineConfig.Builder().setBaseUrl("https://example.com/styles/").build();

		String in = "<p>No styles</p>";
		String out = CssInline.inline(in, cfg);
		assertNotNull(out);
		assertTrue(out.contains("<p>No styles</p>"));
	}

	@Test
	void invalidBaseUrlThrows() {
		CssInlineConfig cfg = new CssInlineConfig.Builder().setBaseUrl("not a url").build();

		CssInlineException ex = assertThrows(CssInlineException.class, () -> CssInline.inline("<p>Hi</p>", cfg));
		assertEquals(ex.getMessage(), "relative URL without a base",
				"Expected URL parse error, got: " + ex.getMessage());
	}

	@Test
	void keepStyleTagsPreserved() {
		String html = "<html><head><style>h1{font-weight:bold}</style></head>" + "<body><h1>Bold</h1></body></html>";

		CssInlineConfig cfg = new CssInlineConfig.Builder().setKeepStyleTags(true).build();

		String out = CssInline.inline(html, cfg);
		assertTrue(out.contains("<style>h1{font-weight:bold}</style>"), "Expected to keep original style tags");
		assertTrue(out.contains("style=\"font-weight: bold;\""));
	}

	@Test
	void negativeCacheSizeThrows() {
		IllegalArgumentException ex = assertThrows(IllegalArgumentException.class,
				() -> new CssInlineConfig.Builder().setCacheSize(-1));
		assertEquals("Cache size must be non-negative, got: -1", ex.getMessage());
	}

	@Test
	void zeroOrNegativePreallocateCapacityThrows() {
		IllegalArgumentException ex1 = assertThrows(IllegalArgumentException.class,
				() -> new CssInlineConfig.Builder().setPreallocateNodeCapacity(0));
		assertEquals("Preallocate node capacity must be positive, got: 0", ex1.getMessage());

		IllegalArgumentException ex2 = assertThrows(IllegalArgumentException.class,
				() -> new CssInlineConfig.Builder().setPreallocateNodeCapacity(-5));
		assertEquals("Preallocate node capacity must be positive, got: -5", ex2.getMessage());
	}

	@Test
	void validConfigurationBuilds() {
		assertDoesNotThrow(() -> {
			CssInlineConfig cfg = new CssInlineConfig.Builder().setCacheSize(0).setCacheSize(100)
					.setPreallocateNodeCapacity(1).build();
		});
	}

  @Test
  void inlineFragmentBasic() {
    String fragment = """
            <main>
            <h1>Hello</h1>
            <section>
            <p>who am i</p>
            </section>
            </main>""";

    String css = """
            p {
                color: red;
            }

            h1 {
                color: blue;
            }
            """;

    String result = CssInline.inlineFragment(fragment, css);

    assertTrue(result.contains("h1 style=\"color: blue;\""), "Should inline h1 color");
    assertTrue(result.contains("p style=\"color: red;\""), "Should inline p color");
    assertFalse(result.contains("<html>"), "Should not add html wrapper");
    assertFalse(result.contains("<head>"), "Should not add head wrapper");
  }

  @Test
  void inlineFragmentWithConfig() {
      String fragment = "<div><h1>Test</h1></div>";
      String css = "h1 { color: blue; font-size: 16px; }";

      CssInlineConfig config = new CssInlineConfig.Builder()
          .setExtraCss("div { margin: 10px; }")
          .build();

      String result = CssInline.inlineFragment(fragment, css, config);

      assertTrue(result.contains("h1 style=\"color: blue;font-size: 16px;\""),
          "Should inline h1 styles");
      assertTrue(result.contains("div style=\"margin: 10px;\""),
          "Should apply extra CSS");
  }

  @Test
  void inlineFragmentEmpty() {
      String result = CssInline.inlineFragment("", "");

      assertEquals("", result, "Empty fragment and CSS should return empty string");
  }
}

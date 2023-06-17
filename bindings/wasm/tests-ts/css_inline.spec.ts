import { expect } from "chai";
import { inline } from "../pkg/css_inline";

describe("CSS inliner", () => {
  describe("default inlining", () => {
    it("h1 style should be applied", function () {
      expect(
        inline(
          "<html><head><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>"
        )
      ).to.equal(
        '<html><head></head><body><h1 style="color:red;">Test</h1></body></html>'
      );
    });
    it("style tag is kept", function () {
      expect(
        inline(
          "<html><head><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
          { keep_style_tags: true }
        )
      ).to.equal(
        '<html><head><style>h1 { color:red; }</style></head><body><h1 style="color:red;">Test</h1></body></html>'
      );
    });
  });
});

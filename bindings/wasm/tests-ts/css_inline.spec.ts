import { expect } from "chai";
import { inline } from "../pkg/css_inline";

describe("CSS inliner", () => {
  describe("default inlining", () => {
    it("h1 style should be applied", function () {
      expect(
        inline(
          "<html><head><title>Test</title><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>"
        )
      ).to.equal(
        '<html><head><title>Test</title><style>h1 { color:red; }</style></head><body><h1 style="color:red;">Test</h1></body></html>'
      );
    });
    it("style tag is removed", function () {
      expect(
        inline(
          "<html><head><title>Test</title><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
          { remove_style_tags: false }
        )
      ).to.equal(
        '<html><head><title>Test</title><style>h1 { color:red; }</style></head><body><h1 style="color:red;">Test</h1></body></html>'
      );
    });
  });
});

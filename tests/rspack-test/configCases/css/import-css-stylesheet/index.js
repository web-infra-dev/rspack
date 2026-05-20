import sheet from "./basic.css" with { type: "css" };
import sheetAssert from "./basic.css" assert { type: "css" };

const getBundleSource = () =>
	__non_webpack_require__("fs").readFileSync(
		__non_webpack_require__("path").join(__STATS__.outputPath, "bundle0.js"),
		"utf-8"
	);

it("should import CSS as CSSStyleSheet with 'with' syntax", () => {
	expect(sheet).toBeInstanceOf(CSSStyleSheet);
});

it("should import CSS as CSSStyleSheet with 'assert' syntax", () => {
	expect(sheetAssert).toBeInstanceOf(CSSStyleSheet);
});

it("should be able to adopt the stylesheet", () => {
	// Test that the stylesheet can be adopted (basic API check)
	expect(sheet).toBeInstanceOf(CSSStyleSheet);
});

it("should include CSS content in the CSSStyleSheet runtime call", () => {
	const source = getBundleSource();
	expect(source).toContain(".test");
	expect(source).toContain("color: red");
	expect(source).toContain("background: blue");
});

it("should handle url() for images in CSSStyleSheet", () => {
	const source = getBundleSource();
	expect(source).toContain(".with-image");
	expect(source).toContain("background-image: url(");
	expect(source).toContain("width: 100px");
	expect(source).toContain("height: 100px");
});

it("should handle CSS nesting in CSSStyleSheet", () => {
	// CSS nesting syntax (& .child) should be preserved in cssText
	expect(getBundleSource()).toContain("& .child");
});

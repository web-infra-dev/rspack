import "./index.css";

const css = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__non_webpack_require__("path").resolve(__dirname, "bundle0.css"), "utf-8"));

it("should use high-level syntax", () => {
	expect(css.includes("-webkit-")).toBe(false);
	expect(css.includes("-moz-")).toBe(false);
});

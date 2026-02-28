import "./index.css";

it("should use high-level css syntax", () => {
	const css = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__non_webpack_require__("path").resolve(__dirname, "bundle0.css"), "utf-8"));
	expect(css.includes("-webkit-")).toBe(false);
	expect(css.includes("-moz-")).toBe(false);
});

it("should use high-level js syntax", () => {
	const js = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));
	const a = [1, 2, 3];
	const b = [...a];
	expect(a).toEqual(b);
	expect(js.includes(".".repeat(3))).toBe(true);
});

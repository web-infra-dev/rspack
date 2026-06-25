import "./index.css";

it("should use high-level css syntax", () => {
	const css = /** @type {string} */ (require("fs").readFileSync(require("path").resolve(__dirname, "bundle0.css"), "utf-8"));
	expect(css.includes("-webkit-")).toBe(false);
	expect(css.includes("-moz-")).toBe(false);
});

it("should use high-level js syntax", () => {
	const js = /** @type {string} */ (require("fs").readFileSync(__filename, "utf-8"));
	const a = [1, 2, 3];
	const b = [...a];
	expect(a).toEqual(b);
	expect(js.includes(".".repeat(3))).toBe(true);
});

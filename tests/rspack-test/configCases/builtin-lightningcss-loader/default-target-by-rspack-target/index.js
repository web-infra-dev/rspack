import "./index.css";

const css = /** @type {string} */ (require("fs").readFileSync(require("path").resolve(__dirname, "bundle0.css"), "utf-8"));

it("should use high-level syntax", () => {
	expect(css.includes("-webkit-")).toBe(false);
	expect(css.includes("-moz-")).toBe(false);
});

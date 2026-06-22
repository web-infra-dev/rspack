import "./index.css";

const css = /** @type {string} */ (require("fs").readFileSync(require("path").resolve(__dirname, "bundle0.css"), "utf-8"));

it("should use low-level syntax", () => {
	expect(css.includes("-webkit-")).toBe(true);
	expect(css.includes("-moz-")).toBe(false);
});

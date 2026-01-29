import "./index.css";

it("should override to high-level css syntax", function () {
	const css = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__non_webpack_require__("path").resolve(__dirname, "bundle0.css"), "utf-8"));
	expect(css.includes("-webkit-")).toBe(false);
	expect(css.includes("-moz-")).toBe(false);
});

it("should override to high-level js syntax", async function () {
	const { deopt } = await import("./deopt");
	const js = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));
	const foo = deopt("foo");
	expect(foo).toBe("foo");
	expect(js.includes(["let", "{deopt:e}", "=", "await"].join())).toBe(true);
});

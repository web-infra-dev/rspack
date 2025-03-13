require("./lib/index.css");
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should compile css url in multiple runtime", async function () {
	await import("./index.css");
	const css = fs.readFileSync(path.resolve(__dirname, "bundle.css"), "utf-8");
	const a = /a: url\((.*)\);/.exec(css)[1];
	expect(a.startsWith("./")).toBe(false);
	expect(a.includes("./img.png")).toBe(false);
	expect(a.endsWith(".png")).toBe(true);
});

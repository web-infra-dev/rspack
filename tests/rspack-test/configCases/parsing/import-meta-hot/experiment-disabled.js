const fs = require("fs");

it("should leave import.meta.hot disabled by default", () => {
	expect(typeof import.meta.hot).toBe("undefined");
	expect(typeof import.meta.webpackHot).toBe("object");

	const source = fs.readFileSync(__filename, "utf-8");
	const hotContextNeedle = [".hmr", "H("].join("");
	expect(source).not.toContain(hotContextNeedle);
});

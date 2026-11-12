import "./scene.gltf";

it("should not add a source map to a text asset that is not JS or CSS", () => {
	const fs = require("fs");
	const path = require("path");
	const assetPath = path.join(__dirname, "text/scene.gltf");
	const content = fs.readFileSync(assetPath, "utf-8");
	expect(content).not.toMatch(/sourceMappingURL/);
	expect(() => JSON.parse(content)).not.toThrow();
	expect(fs.existsSync(assetPath + ".map")).toBe(false);
});

it("should still add a source map to the JS bundle", () => {
	const fs = require("fs");
	const source = fs.readFileSync(__filename, "utf-8");
	expect(source.trimEnd().endsWith("//# sourceMappingURL=bundle0.js.map")).toBe(true);
	expect(fs.existsSync(__filename + ".map")).toBe(true);
});

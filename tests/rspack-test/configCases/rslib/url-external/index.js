const fs = require("fs");
const path = require("path");

it("should keep the external request in new URL() instead of emitting an asset", () => {
	const output = fs.readFileSync(path.resolve(__dirname, "main.mjs"), "utf-8");

	expect(output).toContain('new URL("./mod.js", import.meta.url)');
	expect(output).toContain('new URL("./nested/other.js", import.meta.url)');
	// the externalized target must not be pulled into the module graph
	expect(output).not.toContain("RSPACK_AUTO_URL_STATIC_PLACEHOLDER_");
	expect(output).not.toMatch(/import .*"\.\/mod\.js"/);
});

const fs = require("fs");
const path = require("path");
const { pathToFileURL } = require("url");

it("should render static source phase external binding without default access", async () => {
	const outputPath = path.resolve(__dirname, "main.mjs");
	const output = fs.readFileSync(outputPath, "utf-8");

	expect(output).toMatch(
		/import source __rspack_external_.+ from "\.\/add\.wasm";/
	);
	expect(output).not.toContain('["default"]');
	expect(output).not.toContain(".default");

	fs.copyFileSync(
		path.resolve(__TEST_SOURCE_PATH__, "rslib/static-source-phase-external/add.wasm"),
		path.resolve(__dirname, "add.wasm")
	);

	const mod = await import(`${pathToFileURL(outputPath).href}?t=${Date.now()}`);
	expect(mod.add(1, 2)).toBe(3);
});

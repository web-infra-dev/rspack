require("./App");

it("`module` should be enabled by default", async () => {
	const path = require("path");
	const fs = require("fs");

	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const app = fs.readFileSync(path.resolve(CONTEXT, "./App.jsx"), "utf-8");
	const map = JSON.parse(source);
	let sourceUrl = source => `webpack:///${source}`;
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		sourceUrl = source => `rspack:///${source}`;
	}
	const appSourceIndex = map.sources.indexOf(sourceUrl("./App.jsx"));
	expect(appSourceIndex).toBeGreaterThanOrEqual(0);
	expect(map.sourcesContent[appSourceIndex]).toEqual(app);
});

require("./App");

it("`module` should be enabled by default", async () => {
	const path = require("path");
	const fs = require("fs");

	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const app = fs.readFileSync(path.resolve(CONTEXT, "./App.jsx"), "utf-8");
	const map = JSON.parse(source);
	const appSourceIndex = map.sources.indexOf("webpack:///./App.jsx")
	expect(appSourceIndex).toBeGreaterThanOrEqual(0);
	expect(map.sourcesContent[appSourceIndex]).toEqual(app);
});

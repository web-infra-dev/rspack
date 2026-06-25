const fs = require("fs");
const path = require("path");

it("should honor webpackIgnore when preserved by annotation comments", () => {
	const source = fs.readFileSync(path.join(__dirname, "bundle0.js"), "utf-8");

	expect(source).toMatch(`import(/* @preserve webpackIgnore: true */ url)`);
	expect(source).toMatch(`import(/* @license webpackIgnore: true */ url)`);
});

globalThis.__issue14533__ = {
	loadPreserve(url) {
		return import(/* @preserve webpackIgnore: true */ url);
	},
	loadLicense(url) {
		return import(/* @license webpackIgnore: true */ url);
	}
};

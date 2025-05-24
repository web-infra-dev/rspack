const fs = require('fs');
const path = require('path');

it("Insert module path names with concatenateModules", () => {
	const sourceDir = path.resolve(__dirname, "../../../../");
	const content = fs.readFileSync(path.resolve(__dirname, "modulePathName.js"), "utf-8");
	expect(content).toContain(`const foo_filename = '${path.resolve(sourceDir, "./configCases/plugins/rstest/src/foo.js")}'`);
	expect(content).toMatch(`const bar_dirname = '${path.resolve(sourceDir, "./configCases/plugins/rstest/src")}'`);
});

it("Insert module path names without concatenateModules", () => {
	const sourceDir = path.resolve(__dirname, "../../../../");
	const content = fs.readFileSync(path.resolve(__dirname, "modulePathNameWithoutConcatenate.js"), "utf-8");
	expect(content).toContain(`const __filename = '${path.resolve(sourceDir, "./configCases/plugins/rstest/src/foo.js")}'`);
	expect(content).toMatch(`const __dirname = '${path.resolve(sourceDir, "./configCases/plugins/rstest/src")}'`);

	expect(content.match(/const __dirname = /g).length).toBe(2);
	expect(content.match(/const __filename = /g).length).toBe(2);
});

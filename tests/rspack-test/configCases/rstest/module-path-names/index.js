const fs = require("fs");
const path = require("path");

const sourceDir = path.resolve(__dirname, "../../../../configCases/rstest/module-path-names/src");

it("Insert module path names with concatenateModules", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "modulePathName.js"), "utf-8");
	// __dirname and __filename renamed after concatenation
	expect(content).toContain(`const foo_filename = ${JSON.stringify(path.resolve(sourceDir, "./foo.js"))}`);
	expect(content).toContain(`const foo_dirname = ${JSON.stringify(path.resolve(sourceDir, "."))}`);

	expect(content).toContain(`const metaFile1 = ${JSON.stringify(path.resolve(sourceDir, "./foo.js"))}`)
	expect(content).toContain(`const metaDir1 = ${JSON.stringify(path.resolve(sourceDir, "."))}`)

	expect(content).toContain(`const typeofMetaDir = 'string'`);
	expect(content).toContain(`const typeofMetaFile = 'string'`);
});

it("Insert module path names without concatenateModules", () => {
	const content = fs.readFileSync(path.resolve(__dirname, "modulePathNameWithoutConcatenate.js"), "utf-8");
	expect(content).toContain(`const __filename = ${JSON.stringify(path.resolve(sourceDir, "./foo.js"))}`);
	expect(content).toContain(`const __dirname = ${JSON.stringify(path.resolve(sourceDir, "."))}`);
	expect(content.match(/const __dirname = /g).length).toBe(2);
	expect(content.match(/const __filename = /g).length).toBe(2);

	expect(content).toContain(`const metaFile1 = ${JSON.stringify(path.resolve(sourceDir, "./foo.js"))}`)
	expect(content).toContain(`const metaDir1 = ${JSON.stringify(path.resolve(sourceDir, "."))}`)

	expect(content).toContain(`const typeofMetaDir = 'string'`);
	expect(content).toContain(`const typeofMetaFile = 'string'`);
});

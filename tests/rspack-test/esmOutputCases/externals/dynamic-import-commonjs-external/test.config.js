const fs = require("fs");
const path = require("path");

function readOutput(options) {
	return fs
		.readdirSync(options.output.path)
		.filter(file => file.endsWith(".mjs"))
		.map(file => fs.readFileSync(path.join(options.output.path, file), "utf-8"))
		.join("\n");
}

module.exports = {
	findBundle() {
		return [];
	},
	snapshotFileFilter() {
		return false;
	},
	afterExecute(options) {
		const source = readOutput(options);

		expect(source).not.toMatch(/import\s*\(\s*["']os["']\s*\)/);
		expect(source).toMatch(/module\.createRequire\(import\.meta\.url\)\(\s*["']os["']\s*\)/);
		expect(source).not.toContain("external_os_namespaceObject");
		expect(source).not.toContain("Promise.resolve(external_os_namespaceObject)");
		expect(source).not.toContain('__webpack_require__.t(require("os"), 22)');
		expect(source).toContain(
			'__webpack_require__.t(module.createRequire(import.meta.url)("os"), 22)'
		);
	}
};

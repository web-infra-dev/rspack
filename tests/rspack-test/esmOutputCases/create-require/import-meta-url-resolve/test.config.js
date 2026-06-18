const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8"
		);

		expect(source).not.toContain("file://");
		expect(source).not.toContain("createRequire('<ROOT>");
		expect(source).not.toContain('createRequire("<ROOT>');
		expect(source).not.toContain(
			"external_node_module_namespaceObject.createRequire(import.meta.url)"
		);
		expect(source).toContain("/* createRequire() */ undefined");
		expect(source).toContain("__webpack_require__(");
		expect(source).toContain("/*require.resolve*/");
	}
};

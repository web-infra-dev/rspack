import fs from "node:fs";
import path from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.resolve(options.output.path, "main.mjs"),
			"utf-8"
		);

		expect(source).toMatch(/var __rspack_modules\s*=/);
		expect(source).toMatch(
			/__rspack_context\.r = __rspack_require;\n\n\(function\(\) \{\n\/\/ rspack\/runtime\/define_property_getters\nvar definePropertyGetters/
		);
		expect(source).toContain("// rspack/runtime/define_property_getters");
		expect(source).toContain("// rspack/runtime/make_namespace_object");
		expect(source).toContain("\n})();\n");
		expect(source).not.toContain("}).call(this);");
		expect(source).not.toMatch(
			/__rspack_context\.r = __rspack_require;\n\nvar (?:hasOwnProperty|definePropertyGetters|makeNamespaceObject)/
		);
	}
};

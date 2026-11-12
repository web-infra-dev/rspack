import fs from "node:fs";
import path from "node:path";

export default {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8"
		);

		expect(source).toContain("file://");
		expect(source).toMatch(/createRequire\)?\(['"]file:\/\//);
		expect(source).toContain("/* createRequire() */ undefined");
		expect(source).toContain("__webpack_require__(");
		expect(source).toContain("/*require.resolve*/");
	}
};

import { createRequire } from "node:module";

const req = createRequire(import.meta.url);

export const value = req("./libCssExtractLoader.js");
export const loader = req.resolve("./libCssExtractLoader.js");

it("should consume createRequire(import.meta.url) like webpack", () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const source = fs.readFileSync(path.join(__dirname, "main.mjs"), "utf-8");
	const fileUrlScheme = "file:" + "//";
	const normalizedRoot = "<" + "ROOT>";
	const unresolvedCreateRequire =
		"external_node_module_namespaceObject." + "createRequire(import.meta.url)";

	expect(source).not.toContain(fileUrlScheme);
	expect(source).not.toContain("createRequire('" + normalizedRoot);
	expect(source).not.toContain('createRequire("' + normalizedRoot);
	expect(source).not.toContain(unresolvedCreateRequire);
	expect(source).toContain("/* createRequire() */ undefined");
	expect(source).toContain("__webpack_require__(");
	expect(source).toContain("/*require.resolve*/");
	expect(value).toBe("loader");
	expect(loader).toBe("./libCssExtractLoader.js");
});

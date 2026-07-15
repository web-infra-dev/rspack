import { createRequire } from "node:module";
import { defaultUnknown } from "./default-require.js";
import { exportedRequire } from "./exported-require.js";
import { namespaceUnknown } from "./namespace-require.js";
import {
	getNestedRequire,
	getRequire
} from "./pre-declared-require.js";
import { load } from "./pre-declared-require-call.js";

const req = createRequire(import.meta.url);
const requireWithUnknownMember = createRequire(import.meta.url);
const requireAsValue = createRequire(import.meta.url);

const identity = value => value;

export const value = req("./libCssExtractLoader.js");
export const loader = req.resolve("./libCssExtractLoader.js");
export const unknownMember = requireWithUnknownMember.unknown;
export const requireAsValueType = typeof identity(requireAsValue);

it("should consume createRequire(import.meta.url) like webpack", async () => {
	const fs = await import(/* webpackIgnore: true */ "node:fs");
	const path = await import(/* webpackIgnore: true */ "node:path");
	const source = fs.readFileSync(path.join(__dirname, "main.mjs"), "utf-8");
	const fileUrlScheme = "file:" + "//";
	const normalizedRoot = "<" + "ROOT>";
	const runtimeCreateRequire =
		"(0,external_node_module_namespaceObject." + "createRequire)(import.meta.url)";

	expect(source).not.toContain(fileUrlScheme);
	expect(source).not.toContain("createRequire('" + normalizedRoot);
	expect(source).not.toContain('createRequire("' + normalizedRoot);
	expect(source.split(runtimeCreateRequire)).toHaveLength(6);
	expect(source).toContain("/* createRequire() */ undefined");
	expect(source).toContain("__webpack_require__(");
	expect(source).toContain("/*require.resolve*/");
	expect(value).toBe("loader");
	expect(loader).toBe("./libCssExtractLoader.js");
	expect(unknownMember).toBe(undefined);
	expect(namespaceUnknown).toBe(undefined);
	expect(defaultUnknown).toBe(undefined);
	expect(requireAsValueType).toBe("function");
	expect(exportedRequire.resolve("path")).toBe("path");
	expect(getRequire().resolve("path")).toBe("path");
	expect(getNestedRequire().resolve("path")).toBe("path");
	expect(load()).toBe("loader");
});

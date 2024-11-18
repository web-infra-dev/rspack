const fs = require("fs");
const path = require("path");

it("modern-module-dynamic-import-runtime", () => {
	const initialChunk = fs.readFileSync(path.resolve(__dirname, "main.js"), "utf-8");
	const asyncChunk = fs.readFileSync(path.resolve(__dirname, "async.js"), "utf-8");

	expect(initialChunk).toContain('import * as __WEBPACK_EXTERNAL_MODULE_lit_alias__ from "lit-alias"');
	expect(initialChunk).toContain('import * as __WEBPACK_EXTERNAL_MODULE_svelte_alias__ from "svelte-alias"');
	expect(initialChunk).toContain('import * as __WEBPACK_EXTERNAL_MODULE_react_alias__ from "react-alias"');
	expect(initialChunk).toContain('import * as __WEBPACK_EXTERNAL_MODULE_angular_alias__ from "angular-alias"');
	expect(initialChunk).toContain('const reactNs = await import("react-alias")');
	expect(initialChunk).toContain('const vueNs = await import("vue-alias")');
	expect(initialChunk).toContain('const jqueryNs = await import("jquery-alias", { with: {"type":"url"} })');

	expect(asyncChunk).toContain('const litNs = await import("lit-alias")');
	expect(asyncChunk).toContain('const solidNs = await import("solid-alias")');
});

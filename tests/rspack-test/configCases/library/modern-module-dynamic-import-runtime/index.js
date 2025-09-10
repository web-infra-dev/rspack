const fs = require("fs");
const path = require("path");

it("modern-module-dynamic-import-runtime", () => {
	const initialChunk = fs.readFileSync(path.resolve(__dirname, "main.js"), "utf-8");
	const asyncChunk = fs.readFileSync(path.resolve(__dirname, "async.js"), "utf-8");


	expect(initialChunk).toContain('import { react } from "react-alias"');
	expect(initialChunk).toContain('import { angular } from "angular-alias";');
	expect(initialChunk).toContain('const reactNs = await import("react-alias")');
	expect(initialChunk).toContain('const vueNs = await import("vue-alias")');
	expect(initialChunk).toContain('const jqueryNs = await import("jquery-alias", { with: {"type":"url"} })');
	expect(initialChunk).toContain(`const reactNs2 = await import(/* 123 */ // 456
/*webpackChunkName: 'useless'*/ "react-alias")`)

	expect(asyncChunk).toContain('import { lit } from "lit-alias"');
	expect(asyncChunk).toContain('import { svelte } from "svelte-alias"');
	expect(asyncChunk).toContain('const litNs = await import("lit-alias")');
	expect(asyncChunk).toContain('const solidNs = await import("solid-alias")');

	const initialChunk2 = fs.readFileSync(path.resolve(__dirname, "main2.js"), "utf-8");
	expect(initialChunk2).not.toContain('__webpack_require__');
});

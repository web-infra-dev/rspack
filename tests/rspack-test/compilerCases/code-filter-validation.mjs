import { rspack } from "@rspack/core";
import { fileURLToPath } from "node:url";

const entry = fileURLToPath(
	new URL("../configCases/loader/code-filter/direct.js", import.meta.url)
);
const loader = fileURLToPath(
	new URL("../configCases/loader/code-filter/empty-producer-loader.mjs", import.meta.url)
);

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
	description: "rejects loader code filters other than strings and RegExps",
	async build() {},
	async check({ context }) {
		for (const code of [
			[/marker/],
			{ include: /marker/ },
			{ include: [/marker/] },
			{ exclude: [/marker/] },
			() => true,
			42,
			null
		]) {
			const error = await new Promise(resolve => {
				rspack(
					{
						mode: "development",
						entry,
						output: { path: context.getDist("code-filter-validation") },
						module: {
							rules: [{ use: [{ loader, filter: { code } }] }]
						}
					},
					resolve
				);
			});
			expect(error?.message).toMatch(/Expected a string or RegExp/);
		}
	}
};

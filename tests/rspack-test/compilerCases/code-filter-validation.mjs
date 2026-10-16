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
	description: "rejects non-RegExp loader code filters",
	async build() {},
	async check({ context }) {
		for (const code of [
			"marker",
			[/marker/],
			{ include: "marker" },
			{ include: [/marker/] },
			{ exclude: [/marker/] }
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
			expect(error?.message).toMatch(/Expected a RegExp/);
		}
	}
};

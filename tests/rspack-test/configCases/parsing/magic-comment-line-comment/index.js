import { readdirSync } from "node:fs";
import { load } from "./load.tsx";

it("should keep webpackChunkName before a same-line line comment", async () => {
	// prettier-ignore
	const { c } = await import(/* webpackChunkName: "line-after" */ // a line comment
		"./chunk?line-after");

	expect(c).toBe("ok");
	expect(readdirSync(__dirname)).toContain("line-after.js");
});

it("should keep webpackChunkName when swc-loader moves a line comment onto the same line", async () => {
	expect(await load()).toBe("ok");
	expect(readdirSync(__dirname)).toContain("line-after-await.js");
});

it("should keep webpackChunkName with other line and block comment layouts", async () => {
	const modules = await Promise.all([
		import(/* webpackChunkName: "plain" */ "./chunk?plain"),
		import(
			// a line comment
			/* webpackChunkName: "line-before" */
			"./chunk?line-before"
		),
		import(
			/* webpackChunkName: "block-after" */
			/* another block */
			"./chunk?block-after"
		),
		import(
			/* webpackPrefetch: true */
			// eslint-disable-next-line no-restricted-imports
			/* webpackChunkName: "two-blocks-line-between" */
			"./chunk?two-blocks-line-between"
		)
	]);

	expect(modules.map(({ c }) => c)).toEqual(["ok", "ok", "ok", "ok"]);
	expect(readdirSync(__dirname)).toEqual(
		expect.arrayContaining([
			"plain.js",
			"line-before.js",
			"block-after.js",
			"two-blocks-line-between.js"
		])
	);
});

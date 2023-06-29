import { test, expect } from "@/fixtures";

test("asset emitted hook should only emit modified assets", async ({
	page,
	fileAction,
	rspack
}) => {
	let assets: string[] = [];
	rspack.compiler.hooks.assetEmitted.tap("test", function (name) {
		if (name.includes(".hot-update.")) {
			return;
		}
		assets.push(name);
	});

	expect(await page.textContent("#root")).toBe("__ROOT_TEXT____FOO_VALUE__");

	// update js file
	fileAction.updateFile("src/index.js", content => {
		return content.replace("__ROOT_TEXT__", "__OTHER_TEXT__");
	});
	await rspack.waitingForHmr(async () => {
		const text = await page.textContent("#root");
		return text === "__OTHER_TEXT____FOO_VALUE__";
	});
	expect(assets).toEqual(["main.js"]);

	// reset assets
	assets.length = 0;

	// update css file
	fileAction.updateFile("src/foo.js", content => {
		return content.replace("__FOO_VALUE__", "__VALUE__");
	});
	await rspack.waitingForHmr(async () => {
		const text = await page.textContent("#root");
		return text === "__OTHER_TEXT____VALUE__";
	});
	// main.js contains runtime module, so it should also emit
	expect(assets).toEqual(["src_foo_js.js", "main.js"]);
});

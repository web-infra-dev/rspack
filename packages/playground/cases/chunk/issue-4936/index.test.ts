import { test } from "@/fixtures";

test("should load success", async ({ page, fileAction, rspack }) => {
	await rspack.waitUntil(async function () {
		return (await page.title()) === "123";
	});

	// use webpackChunkName
	fileAction.updateFile(
		"./src/index.js",
		() => `
import(/* webpackChunkName: "bar" */'./app').then((m) => {
	m.title('456')
});`
	);
	await rspack.waitUntil(async function () {
		return (await page.title()) === "456";
	});
});

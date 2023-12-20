import { expect, test } from "@/fixtures";

test("should load success", async ({ page, fileAction }) => {
	await expect(page).toHaveTitle("123");

	// use webpackChunkName
	fileAction.updateFile(
		"./src/index.js",
		() => `
import(/* webpackChunkName: "bar" */'./app').then((m) => {
	m.title('456')
});`
	);

	await expect(page).toHaveTitle("456");
});

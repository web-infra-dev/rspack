import { test, expect } from "@/fixtures";

test("render should work", async ({ page }) => {
	await expect(page.locator(".header")).toHaveText("Hello World");
});

test("class component hmr should work", async ({ page, fileAction }) => {
	await expect(page.locator("button")).toHaveText("10");
	await page.click("button");
	await expect(page.locator("button")).toHaveText("11");
	await expect(page.locator(".placeholder")).toHaveText("__PLACE_HOLDER__");
	fileAction.updateFile("src/App.jsx", content =>
		content.replace("__PLACE_HOLDER__", "__EDITED__")
	);
	await expect(page.locator(".placeholder")).toHaveText("__EDITED__");
	// class component will not keep local status
	await expect(page.locator("button")).toHaveText("10");
});

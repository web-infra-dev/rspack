import { test, expect } from "@/fixtures";

test("render should work", async ({ page }) => {
	expect(await page.textContent(".header")).toBe("Hello World");
	expect(await page.textContent("#lazy-component")).toBe("Lazy Component");
});

test("hmr should work", async ({ page, fileAction }) => {
	await expect(page.locator("button")).toHaveText("10");
	await page.click("button");
	await expect(page.locator("button")).toHaveText("11");
	await expect(page.locator(".placeholder")).toHaveText("__PLACE_HOLDER__");
	fileAction.updateFile("src/App.jsx", content =>
		content.replace("__PLACE_HOLDER__", "__EDITED__")
	);
	await expect(page.locator(".placeholder")).toHaveText("__EDITED__");
	await expect(page.locator("button")).toHaveText("11");
});

test("context+component should work", async ({ page, fileAction }) => {
	await expect(page.locator("#context")).toHaveText("context-value");
	await page.click("#context");
	await expect(page.locator("#context")).toHaveText("context-value-click");
	fileAction.updateFile("src/CountProvider.jsx", content =>
		content.replace("context-value", "context-value-update")
	);
	await expect(page.locator("#context")).toHaveText("context-value-update");
});

test("ReactRefreshFinder should work", async ({ page }) => {
	await expect(page.locator("#nest-function")).toHaveText("nest-function");
});

test("update same export name from different module should work", async ({
	page,
	fileAction
}) => {
	await expect(page.locator(".same-export-name1")).toHaveText("__NAME_1__");
	await expect(page.locator(".same-export-name2")).toHaveText("__NAME_2__");
	fileAction.updateFile("src/SameExportName1.jsx", content =>
		content.replace("__NAME_1__", "__name_1__")
	);
	await expect(page.locator(".same-export-name1")).toHaveText("__name_1__");
	await expect(page.locator(".same-export-name2")).toHaveText("__NAME_2__");
	fileAction.updateFile("src/SameExportName2.jsx", content =>
		content.replace("__NAME_2__", "__name_2__")
	);
	await expect(page.locator(".same-export-name1")).toHaveText("__name_1__");
	await expect(page.locator(".same-export-name2")).toHaveText("__name_2__");
});

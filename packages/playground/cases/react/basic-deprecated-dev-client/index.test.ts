import { test, expect } from "@/fixtures";

test("render should work", async ({ page }) => {
	expect(await page.textContent(".header")).toBe("Hello World");
});

test("hmr should work", async ({ page, fileAction, rspack }) => {
	expect(await page.textContent("button")).toBe("10");
	await page.click("button");
	expect(await page.textContent("button")).toBe("11");
	expect(await page.textContent(".placeholder")).toBe("__PLACE_HOLDER__");
	fileAction.updateFile("src/App.jsx", content =>
		content.replace("__PLACE_HOLDER__", "__EDITED__")
	);
	await rspack.waitingForHmr(async function () {
		return (await page.textContent(".placeholder")) === "__EDITED__";
	});
	expect(await page.textContent("button")).toBe("11");
});

test("context+component should work", async ({ page, fileAction, rspack }) => {
	expect(await page.textContent("#context")).toBe("context-value");
	await page.click("#context");
	expect(await page.textContent("#context")).toBe("context-value-click");
	fileAction.updateFile("src/CountProvider.jsx", content =>
		content.replace("context-value", "context-value-update")
	);
	await rspack.waitingForHmr(async function () {
		return (await page.textContent("#context")) === "context-value-update";
	});
});

test("ReactRefreshFinder should work", async ({ page }) => {
	expect(await page.textContent("#nest-function")).toBe("nest-function");
});

test("update same export name from different module should work", async ({
	page,
	fileAction,
	rspack
}) => {
	expect(await page.textContent(".same-export-name1")).toBe("__NAME_1__");
	expect(await page.textContent(".same-export-name2")).toBe("__NAME_2__");
	fileAction.updateFile("src/SameExportName1.jsx", content =>
		content.replace("__NAME_1__", "__name_1__")
	);
	await rspack.waitingForHmr(async function () {
		return (await page.textContent(".same-export-name1")) === "__name_1__";
	});
	expect(await page.textContent(".same-export-name2")).toBe("__NAME_2__");
	fileAction.updateFile("src/SameExportName2.jsx", content =>
		content.replace("__NAME_2__", "__name_2__")
	);
	await rspack.waitingForHmr(async function () {
		return (await page.textContent(".same-export-name2")) === "__name_2__";
	});
	expect(await page.textContent(".same-export-name1")).toBe("__name_1__");
});

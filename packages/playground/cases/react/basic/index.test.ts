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

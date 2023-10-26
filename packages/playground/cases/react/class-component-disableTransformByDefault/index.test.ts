import { test, expect } from "@/fixtures";

test("render should work", async ({ page }) => {
	expect(await page.textContent(".header")).toBe("Hello World");
});

test("class component hmr should work", async ({
	page,
	fileAction,
	rspack
}) => {
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
	// class component will not keep local status
	expect(await page.textContent("button")).toBe("10");
});

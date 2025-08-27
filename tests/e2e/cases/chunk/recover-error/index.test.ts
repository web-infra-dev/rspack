import { test, expect } from "@/fixtures";

test("should error with invalid syntax", async ({ page, fileAction }) => {
	await expect(page.locator("button")).toHaveText("count is 0");
	await page.click("button");
	await expect(page.locator("button")).toHaveText("count is 1");
	fileAction.updateFile("src/App.jsx", content =>
		content.replace("</div>", "{/* </div> */}")
	);
	await expect(page.locator("#webpack-dev-server-client-overlay")).toHaveCount(
		1
	);
	const brokenCode = fileAction.readDistFile("AppIndex.js")!;
	expect(/throw new Error\(".*Unexpected token. Did you mean `{'}'}`/.test(brokenCode)).toBe(true);
	fileAction.updateFile("src/App.jsx", content =>
		content.replace("{/* </div> */}", "</div>")
	);
	await expect(page.locator("#webpack-dev-server-client-overlay")).toHaveCount(
		0
	);
	const fixedCode = fileAction.readDistFile("AppIndex.js")!;
	expect(/throw new Error\(".*Unexpected token. Did you mean `{'}'}`/.test(fixedCode)).toBe(false);
});

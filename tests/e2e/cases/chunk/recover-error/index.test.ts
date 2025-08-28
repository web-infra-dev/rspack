import { test, expect } from "@/fixtures";
import path from "path";

test("should error with invalid syntax", async ({ page, fileAction, rspack }) => {
	const codePath = path.resolve(rspack.outDir, "AppIndex.js");
	await expect(page.locator("button")).toHaveText("count is 0");
	await page.click("button");
	await expect(page.locator("button")).toHaveText("count is 1");
	fileAction.updateFile("src/App.jsx", content =>
		content.replace("</div>", "{/* </div> */}")
	);
	await expect(page.locator("#webpack-dev-server-client-overlay")).toHaveCount(
		1
	);
	const brokenCode = rspack.compiler.outputFileSystem.readFileSync(codePath, "utf-8");
	expect(/throw new Error\(".*Unexpected token. Did you mean `{'}'}`/.test(brokenCode)).toBe(true);
});

import { test, expect } from "@/fixtures";

test("HMR should work for multiple runtime in the same HTML page", async ({ page, fileAction }) => {
	await expect(page.locator("#a")).toHaveCSS("color", "rgb(255, 0, 0)");
	await expect(page.locator("#b")).toHaveCSS("color", "rgb(0, 0, 255)");
	fileAction.updateFile("./src/a.module.css", content =>
		content.replace("color: rgb(255, 0, 0)", "color: rgb(0, 0, 255)")
	);
	await expect(page.locator("#a")).toHaveCSS("color", "rgb(0, 0, 255)");
	fileAction.updateFile("./src/b.module.css", content =>
		content.replace("color: rgb(0, 0, 255)", "color: rgb(0, 255, 0)")
	);
	await expect(page.locator("#b")).toHaveCSS("color", "rgb(0, 255, 0)");
	fileAction.updateFile("./src/a.module.css", content =>
		content.replace("color: rgb(0, 0, 255)", "color: rgb(0, 255, 0)")
	);
	await expect(page.locator("#a")).toHaveCSS("color", "rgb(0, 255, 0)");
	fileAction.updateFile("./src/b.module.css", content =>
		content.replace("color: rgb(0, 255, 0)", "color: rgb(255, 0, 0)")
	);
	await expect(page.locator("#b")).toHaveCSS("color", "rgb(255, 0, 0)");
});

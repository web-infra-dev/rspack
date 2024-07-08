import { test, expect } from "@/fixtures";

test("should update body css", async ({ page, fileAction, rspack }) => {
	await expect(page.locator("body")).toHaveCSS("display", "block");
	fileAction.updateFile("src/index.css", content =>
		content.replace("block", "flex")
	);
	await expect(page.locator("body")).toHaveCSS("display", "flex");
});

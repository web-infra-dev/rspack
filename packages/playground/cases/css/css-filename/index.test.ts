import { test, expect } from "@/fixtures";

test("should update css with filename", async ({
	page,
	fileAction,
	rspack
}) => {
	await expect(page.locator("body")).toHaveCSS("color", "rgb(255, 0, 0)");
	fileAction.updateFile("src/index.css", content =>
		content.replace("rgb(255, 0, 0)", "rgb(0, 0, 255)")
	);
	await expect(page.locator("body")).toHaveCSS("color", "rgb(0, 0, 255)");
});

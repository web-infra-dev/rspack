import { test, expect } from "@/fixtures";

test("should not throw error for importing empty css files", async ({
	page
}) => {
	await expect(page.locator("#root")).toHaveText("ok");
});

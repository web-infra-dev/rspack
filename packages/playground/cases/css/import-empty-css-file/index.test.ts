import { test, expect } from "@/fixtures";

test("should not throw error for importing empty css files", async ({
	page
}) => {
	expect(await page.textContent("#root")).toBe("ok");
});

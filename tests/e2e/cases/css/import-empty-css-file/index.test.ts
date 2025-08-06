import { test, expect, waitForHmr } from "@/fixtures";

test("should not throw error for importing empty css files", async ({
	page
}) => {
	await waitForHmr(page);
	expect(await page.textContent("#root")).toBe("ok");
});

import { test, expect } from "@/fixtures";

test("should successfully render vue3", async ({ page }) => {
	expect(await page.textContent("h1")).toBe("vue3");
});

test("vue3 hmr", async ({ page, fileAction }) => {
	await page.click("button");
	await expect(page.locator("button")).toHaveText("1");
	fileAction.updateFile("src/App.vue", content =>
		content.replace("vue3", "vue3 hi")
	);
	await expect(page.locator("h1")).toHaveText("vue3 hi");
	// hmr should keep status
	await expect(page.locator("button")).toHaveText("1");
});

// See: https://vuejs.org/guide/typescript/composition-api.html#typing-component-props
test("vue3 should work with component props typing", async ({
	page,
	fileAction
}) => {
	fileAction.updateFile("src/enums.ts", content => content.replace("0", "2"));
	await expect(page.locator("button")).toHaveText("2");
});

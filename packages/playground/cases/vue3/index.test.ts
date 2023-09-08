import { test, expect } from "@/fixtures";

test("should successfully render vue3", async ({ page }) => {
	expect(await page.textContent("h1")).toBe("vue3");
});

test("vue3 hmr", async ({ page, fileAction, rspack }) => {
	await page.click("button");
	expect(await page.textContent("button")).toBe("1");
	fileAction.updateFile("src/App.vue", content =>
		content.replace("vue3", "vue3 hi")
	);
	await rspack.waitingForHmr(async function () {
		return (await page.textContent("h1")) === "vue3 hi";
	});
	// hmr should keep status
	expect(await page.textContent("button")).toBe("1");
});

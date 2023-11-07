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

// See: https://vuejs.org/guide/typescript/composition-api.html#typing-component-props
test("vue3 should work with component props typing", async ({
	page,
	rspack,
	fileAction
}) => {
	fileAction.updateFile("src/enums.ts", content => content.replace("0", "2"));
	await rspack.waitingForHmr(async function () {
		return (await page.textContent("button")) === "2";
	});
});

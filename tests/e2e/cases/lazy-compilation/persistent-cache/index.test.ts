import { expect, test } from "@/fixtures";

test("should load success", async ({ page, rspack }) => {
	await page.getByText("Click me").click();
	let component_count = await page.getByText("Component").count();
	expect(component_count).toBe(1);

	await rspack.reboot();
	await page.reload();

	// trigger other import compile
	await page.getByText("Click me").click();
	await page.waitForEvent('console')
	const dynImported = await page.getByText("dyn imported").count();
	expect(dynImported).toBe(1);
});

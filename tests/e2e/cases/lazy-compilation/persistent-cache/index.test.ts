import { expect, test } from "@/fixtures";

test("should load success", async ({ page, rspack }) => {
	await page.getByText("Click me").click();
	let component_count = await page.getByText("Component").count();
	expect(component_count).toBe(1);

	const responsePromise = page.waitForResponse(
		response =>
			response.url().includes("lazy-compilation-using") &&
			response.request().method() === "GET",
		{ timeout: 5000 }
	);
	await rspack.reboot();
	await page.reload();
	await responsePromise;

	await page.getByText("Click me").click();
	component_count = await page.getByText("Component").count();
	expect(component_count).toBe(1);
});

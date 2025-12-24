import { expect, test } from "@/fixtures";

test("should use custom prefix for lazy compilation", async ({ page }) => {
	// Wait for a request with custom prefix
	const responsePromise = page.waitForResponse(
		response =>
			response.url().includes("/custom-lazy-endpoint-") &&
			response.request().method() === "GET",
		{ timeout: 5000 }
	);

	// Click the button that triggers dynamic import
	await page.getByText("Click me").click();

	// Wait for response with custom prefix
	const response = await responsePromise;
	expect(response.status()).toBe(200);

	// Wait for the component to appear with a more reliable wait
	await page.waitForSelector('div:has-text("Component")', { timeout: 5000 });

	// Check that the component was loaded and displayed
	const component_count = await page.getByText("Component").count();
	expect(component_count).toBe(1);

	// Verify that the request was made using the custom prefix
	expect(response.url()).toContain("/custom-lazy-endpoint-");
});

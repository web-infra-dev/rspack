import { expect, test } from "@/fixtures";

test("should use default prefix for lazy compilation", async ({ page }) => {
	// Wait for a request with default prefix
	const responsePromise = page.waitForResponse(
		response => {
			console.log(`server url: ${response.url()}`)
			return response.url().includes("/lazy-compilation-using-") &&
				response.request().method() === "GET"
		},
		{ timeout: 5000 }
	);

	// Click the button that triggers dynamic import
	await page.getByText("Click me").click();

	// Wait for response with default prefix
	const response = await responsePromise;
	expect(response.status()).toBe(200);

	// Wait for the component to appear with a more reliable wait
	await page.waitForSelector('div:has-text("Component")', { timeout: 5000 });

	// Check that the component was loaded and displayed
	const component_count = await page.getByText("Component").count();
	expect(component_count).toBe(1);

	// Verify that the request was made using the default prefix
	expect(response.url()).toContain("/lazy-compilation-using-");
});

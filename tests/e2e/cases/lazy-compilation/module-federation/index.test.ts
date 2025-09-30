import { expect, test } from "@/fixtures";

test("should load remote and shared success", async ({ page }) => {
	// Click the button that triggers dynamic import
	await page.waitForSelector('button:has-text("Click me")');
	await page.getByText("Click me").click();

	// Wait for the component to appear with a more reliable wait
	await page.waitForSelector('div:has-text("RemoteComponent")');

	// Check that the component was loaded and displayed
	const RemoteComponentCount = await page.getByText("RemoteComponent").count();
	expect(RemoteComponentCount).toBe(1);


	// Check that the shared component was loaded and displayed
	const SharedReactCount = await page.getByText("SharedReact").count();
	expect(SharedReactCount).toBe(1);
});

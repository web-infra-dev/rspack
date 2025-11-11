import { expect, test } from "@playwright/test";

const apps = [
	{ port: 3001, name: "App 1", selector: "h6", text: "Module Federation Demo" },
	{ port: 3002, name: "App 2", selector: "h6", text: "Material UI App" },
	{ port: 3003, name: "App 3", selector: "h6", text: "Styled Components App" },
	{
		port: 3004,
		name: "App 4",
		selector: "h1",
		text: "Hello From Svelte world!"
	}
];

apps.forEach(({ port, name, selector, text }) => {
	test.describe(name, () => {
		test(`build and run ${name}`, async ({ page }) => {
			await page.goto(`http://localhost:${port}`);
			await expect(page.locator(selector, { hasText: text })).toBeVisible();
		});
	});
});

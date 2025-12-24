import { test, expect } from "@/fixtures";

test("tailwindcss should work when modify js file", async ({
	page,
	fileAction,
	rspack
}) => {
	await expect(page.locator("#app")).toHaveClass(/text-2xl/);

	// update
	fileAction.updateFile("src/App.jsx", content => {
		return content.replace("text-2xl", "text-3xl");
	});

	await expect(page.locator("#app")).toHaveClass(/text-3xl/);
});

import { test, expect } from "@/fixtures";

test("tailwindcss should work when modify js file", async ({
	page,
	fileAction,
	rspack
}) => {
	function getAppFontSize() {
		return page.evaluate(() => {
			const app = document.querySelector("#app");
			if (!app) {
				return "";
			}
			return window.getComputedStyle(app).fontSize;
		});
	}

	let appFontSize = await getAppFontSize();
	expect(appFontSize).toBe("24px");

	// update
	fileAction.updateFile("src/App.jsx", content => {
		return content.replace("text-2xl", "text-3xl");
	});

	await expect(page.locator("#app")).toHaveClass(/text-3xl/);

	appFontSize = await getAppFontSize();
	expect(appFontSize).toBe("30px");
});

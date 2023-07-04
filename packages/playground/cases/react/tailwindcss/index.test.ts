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
	await rspack.waitingForHmr(async () => {
		const classNames = await page.getAttribute("#app", "class");
		return classNames?.includes("text-3xl") || false;
	});

	appFontSize = await getAppFontSize();
	expect(appFontSize).toBe("30px");
});

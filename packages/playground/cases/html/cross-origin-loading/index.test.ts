import { test, expect } from "@/fixtures";

test("should set crossOrigin to anonymous for script tag correctly", async ({
	page
}) => {
	const scripts = await page.$$("script");

	const crossOrigins = await Promise.all(
		scripts.map(script => script.getAttribute("crossorigin"))
	);

	expect(crossOrigins).toEqual([null, "anonymous"]);
});

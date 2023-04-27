import { wait } from "../../utils";

test("should set crossOrigin to anonymous for script tag correctly", async () => {
	await wait(50);
	const scripts = await page.$$("script");

	const crossOrigins = await Promise.all(
		scripts.map(script => script.getAttribute("crossorigin"))
	);

	expect(crossOrigins).toEqual([null, "anonymous"]);
});

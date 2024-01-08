import { test, expect } from "@/fixtures";

test("delete file should work", async ({
	page,
	request,
	fileAction,
	rspack
}) => {
	// asset page status
	const statusText = await page.textContent("#root");
	expect(statusText).toBe("__PAGE_RENDER__");
	// asset script content
	const response = await request.get(
		`http://localhost:${rspack.devServer.options.port}/main.js`
	);
	const bodyResponse = (await response.body()).toString();
	expect(bodyResponse).toContain("this is mod1");
	expect(bodyResponse).toContain("this is mod2");
	// mock file delete action
	fileAction.deleteFile("src/mod1.js");
	await expect(page.locator("#root")).toHaveText("__HMR_UPDATED__");
	// asset new script content
	const responseAfterDelete = await request.get(
		`http://localhost:${rspack.devServer.options.port}/main.js`
	);
	const bodyResponseAfterDelete = (await responseAfterDelete.body()).toString();
	expect(bodyResponseAfterDelete).not.toContain("this is mod1");
	expect(bodyResponseAfterDelete).toContain("this is mod2");
});

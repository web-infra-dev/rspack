import { test, expect } from "@/fixtures";

test("html should refresh after reload", async ({
	page,
	fileAction,
	rspack
}) => {
	await expect(page.title()).resolves.toBe("123");
	fileAction.updateFile("./src/index.html", content =>
		content.replace("123", "456")
	);
	await rspack.waitUntil(async () => {
		await page.reload();
		const t2 = await page.title();
		return t2 === "456";
	});
});

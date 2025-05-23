import { test, expect } from "@/fixtures";

test("hmr works for multiple instances of the same runtime", async ({
	page,
	fileAction,
	rspack,
}) => {
	await expect(await page.getByTestId('0')).toHaveText("1");
	await expect(await page.getByTestId('1')).toHaveText("1");

	fileAction.updateFile("src/value.js", content => {
		return content.replace("1", "2");
	});

	await expect(await page.getByTestId('0')).toHaveText("2");
	await expect(await page.getByTestId('1')).toHaveText("2");

	fileAction.updateFile("src/value.js", content => {
		return content.replace("'2'", "Math.random().toString()")
	});

	await expect(await page.getByTestId('0')).not.toHaveText("2");
	await expect(await page.getByTestId('1')).not.toHaveText("2");

	const text0 = await page.getByTestId('0').textContent();
	const text1 = await page.getByTestId('1').textContent();
	await expect(text0).not.toEqual(text1);
});

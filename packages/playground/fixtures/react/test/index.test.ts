import { editFile, waitingUpdate } from "../../utils";

test("render should work", async () => {
	expect(await page.textContent(".header")).toBe("Hello World");
});

test("hmr should work", async () => {
	expect(await page.textContent("button")).toBe("10");
	await page.click("button");
	expect(await page.textContent("button")).toBe("11");
	editFile("src/App.jsx", content =>
		content.replace("__PLACE_HOLDER__", "__EDITED__")
	);
	await waitingUpdate(() => page.textContent(".placeholder"), "__EDITED__");
	expect(await page.textContent("button")).toBe("11");
});

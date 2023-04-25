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

test("context+component should work", async () => {
	expect(await page.textContent("#context")).toBe("context-value");
	await page.click("#context");
	expect(await page.textContent("#context")).toBe("context-value-click");
	editFile("src/CountProvider.jsx", content =>
		content.replace("context-value", "context-value-update")
	);
	await waitingUpdate(
		() => page.textContent("#context"),
		"context-value-update"
	);
});

test("ReactRefreshFinder should work", async () => {
	expect(await page.textContent("#nest-function")).toBe("nest-function");
});

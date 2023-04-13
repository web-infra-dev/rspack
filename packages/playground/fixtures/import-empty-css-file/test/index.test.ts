test("should not throw error for importing empty css files", async () => {
	expect(await page.textContent("#root")).toBe("ok");
});

export {};

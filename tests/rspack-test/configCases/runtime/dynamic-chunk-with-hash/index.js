it("load dynamic chunk with hash", async function () {
	const module = await import("./dynamic");
	expect(module.value).toBe("dynamic");
});

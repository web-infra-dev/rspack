it("load dynamic chunk", async function () {
	const module = await import("./dynamic");
	expect(module.value).toBe(1);
});

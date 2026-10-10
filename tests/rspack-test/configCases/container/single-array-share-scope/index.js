it("should support single-item shareScope array in non-enhanced container", async () => {
	const container = __non_webpack_require__("./container-file.js");
	expect(container).toBeTypeOf("object");
	expect(container.get).toBeTypeOf("function");
	const testFactory = await container.get("./test");
	expect(testFactory).toBeTypeOf("function");
	expect(testFactory()).toBe("test");
});

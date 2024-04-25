it("load dynamic chunk with split commmon chunk", async () => {
	await Promise.all([
		import("./a").then(module => {
			expect(module.default).toBe("a");
			expect(module.common).toBe("common");
		}),
		import("./b").then(module => {
			expect(module.default).toBe("b");
			expect(module.common).toBe("common");
		})
	])
});

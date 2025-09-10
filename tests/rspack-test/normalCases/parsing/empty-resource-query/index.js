it("should detect query strings in dynamic import as a static value 1 ", function () {
	return Promise.all([
		import("./a").then(({ default: a }) => {
			expect(a()).toBe("a");
		}),
		import("./abc").then(({ default: a }) => {
			expect(a()).toBe("abc");
		}),
		import("./a?queryString").then(({ default: a }) => {
			expect(a()).toBe("a?queryString");
		})
	]);
});

it("should detect query strings in dynamic import as a static value 1 ", function () {
	return import("./a?queryString").then(({ default: a }) => {
		expect(a()).toBe("a?queryString?queryString");
	})
});

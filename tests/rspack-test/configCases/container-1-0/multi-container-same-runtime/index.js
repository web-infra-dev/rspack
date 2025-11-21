it("should import the correct modules", () => {
	return import("./bootstrap").then(({ test }) => test(it))
});

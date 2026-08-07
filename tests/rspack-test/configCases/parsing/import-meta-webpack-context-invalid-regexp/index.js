const contextRequire = import.meta.webpackContext(".", {
	regExp: /(?<name>a)|(?<name>b)/,
	recursive: false
});

it("should warn, ignore an invalid regexp and preserve other options", () => {
	expect(contextRequire("./value")).toBe("value");
	expect(contextRequire.keys()).not.toContain("./nested/value.js");
});

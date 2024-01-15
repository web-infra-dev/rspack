it("should throw error instead of panic", () => {
	expect(() => require("./lib-entry")).toThrowError(
		/Helper not defined: \"unregisteredCase\"/
	);
});

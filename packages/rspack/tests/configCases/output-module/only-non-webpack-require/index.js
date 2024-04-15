it("should work with only using __non_webpack_require__ and ES modules", function () {
	const foo = __non_webpack_require__("./mod.js");
	expect(foo).toBe("module text");
});

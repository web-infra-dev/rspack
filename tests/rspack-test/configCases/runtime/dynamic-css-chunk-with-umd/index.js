it("should have lib module exports", function () {
	__non_webpack_require__("./runtime~lib.js");
	__non_webpack_require__("./dynamic_js.js");
	const lib = __non_webpack_require__("./lib.js");
	expect(lib.value).toBe(42);
})

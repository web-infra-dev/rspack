it("should have lib module exports", function () {
	eval("require")("./runtime~lib.js");
	eval("require")("./dynamic_js.js");
	const lib = eval("require")("./lib.js");
	expect(lib.value).toBe(42);
})

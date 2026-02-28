it("context module(sync) + cjs require + binary expression", function () {
	let a = "child/index";
	let module = require("./" + a + ".js");
	expect(module.value).toBe("dynamic");
});

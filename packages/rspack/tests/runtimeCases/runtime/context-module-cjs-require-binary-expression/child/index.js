it("context module(sync) + cjs require + binary expression", function (done) {
	let a = "index";
	let module = require("./child/" + a + ".js");
	expect(module.value).toBe("dynamic");
	done();
});

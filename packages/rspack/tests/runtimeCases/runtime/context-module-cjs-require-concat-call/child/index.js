it("context module(sync) + cjs require + concat call", function (done) {
	let a = "index";
	let module = require("./child/".concat(a, ".js"));
	expect(module.value).toBe("dynamic");
	done();
});

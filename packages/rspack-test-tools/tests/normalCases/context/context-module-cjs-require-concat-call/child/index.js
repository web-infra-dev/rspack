it("context module(sync) + cjs require + concat call", function (done) {
	const a = "index";
	const module = require("./child/".concat(a, ".js"));
	expect(module.value).toBe("dynamic");
	done();
});

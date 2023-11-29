it("context module(sync) + cjs require + tagged template", function (done) {
	let a = "child/index";
	let module = require(String.raw`./${a}.js`);
	expect(module.value).toBe("dynamic");

	let tagFunc = function () {
		return "fail";
	};
	expect(function () {
		require(tagFunc`./${a}.js`);
	}).toThrowError(/Cannot find module/);
	done();
});

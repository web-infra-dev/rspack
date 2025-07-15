it("context module(sync) + cjs require + template literal", function (done) {
	const a = "index";
	const module = require(`./child/${a}.js`);
	expect(module.value).toBe("dynamic");
	done();
});

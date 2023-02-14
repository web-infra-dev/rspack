it("cjs require context module", function (done) {
	let a = "index";
	let module = require(`./child/${a}.js`);
	expect(module.value).toBe("dynamic");
	done();
});

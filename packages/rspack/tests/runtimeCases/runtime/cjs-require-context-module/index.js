it("cjs require context module", function (done) {
	let params = "index";
	debugger;
	let module = require(`./child/${params}.js`);
	expect(module.value).toBe("dynamic");
	done();
});

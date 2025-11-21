it("context module(sync) + cjs require + template literal", function () {
	let a = "index";
	let module = require(`./child/${a}.js`);
	expect(module.value).toBe("dynamic");
});

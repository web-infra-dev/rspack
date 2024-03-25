it("should apply inline loaders before matchResource", function () {
	const foo = require(`a.js!=!loader1?{"foo":"#bar"}!./b.js`);

	expect(foo).toEqual({ foo: "#bar" });
});

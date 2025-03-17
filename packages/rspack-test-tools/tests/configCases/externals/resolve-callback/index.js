it("should allow functions as externals with promise and resolver", function () {
	const result = require("external");
	expect(result.resource).toMatch(/^[a-z]:\\|\//i);
	expect(result.resource).toMatch(/resolve-callback.node_modules.external\.js$/);
	expect(result.esm).toBe(true);
});

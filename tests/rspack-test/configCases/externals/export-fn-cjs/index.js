it("Should work with export a function", function() {
	const myModule = require("module");
	expect(typeof myModule).toBe("function");
	expect(myModule.builtinModules).toBeDefined();
});

it("should work with export a object", function() {
	const myFs = require("fs");
	expect(typeof myFs).toBe("object");
	expect(myFs.readFileSync).toBeDefined();
});

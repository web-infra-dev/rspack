it("should pass change module type to json", () => {
	let result = require("foo.rspack[json]!=!!./loader-test.mjs!./foo.custom");
	expect(result).toEqual({
		hello: "world"
	});
});

it("should pass change module type to json with compatibility", () => {
	let result = require("foo.webpack[json]!=!!./loader-test.mjs!./foo.custom");
	expect(result).toEqual({
		hello: "world"
	});
});


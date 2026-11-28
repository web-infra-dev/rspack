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

it("should use the final webpack suffix when the resource contains webpack earlier", () => {
	const result = require("foo.webpack.bar.webpack[json]!=!!./loader-test.mjs!./foo.custom");
	expect(result).toEqual({
		hello: "world"
	});
});


it("should pass change module type to json", () => {
	let result = require("foo.webpack[json]!=!!./loader-test!./foo.custom");
	expect(result).toEqual({
		hello: "world"
	});
});

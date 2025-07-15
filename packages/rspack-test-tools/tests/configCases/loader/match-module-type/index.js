it("should pass change module type to json", () => {
	const result = require("foo.webpack[json]!=!!./loader-test!./foo.custom");
	expect(result).toEqual({
		hello: "world"
	});
});

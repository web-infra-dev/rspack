it("should emit the correct error for loaders not returning buffer or string", function() {
	expect(() => require("./loader.mjs!./file.js")).toThrowError(
		/Module build failed/
	);
	expect(() => require("./loader.mjs!./pitch-loader.mjs!./file.js")).toThrowError(
		/Module build failed/
	);
});

it("should error loadModule when the referenced module contains errors", () => {
	expect(function() {
		require("./loader.mjs!./a")
	}).toThrowError();
});

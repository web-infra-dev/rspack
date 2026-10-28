it("should allow to access utils in loader", () => {
	expect(require("./loader.mjs!" + __filename)).toEqual({
		request1: "./index.js",
		request2: "./index.js"
	});
});

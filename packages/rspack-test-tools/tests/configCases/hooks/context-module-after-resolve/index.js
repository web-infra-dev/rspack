it("should compile", () => {
	expect(() => {
		["foo.js"].map(file => {
			require("./dir/" + file);
		});
	}).toThrow("Cannot find module './dir'")
});

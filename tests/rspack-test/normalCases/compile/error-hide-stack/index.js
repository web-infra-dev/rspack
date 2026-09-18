it("should hide stack in details", function() {
	expect(function f() {
		require("./loader.mjs!");
	}).toThrowError();
});

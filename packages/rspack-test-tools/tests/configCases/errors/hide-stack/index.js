it("should hide stack in details when emit", function () {
	expect(function f() {
		require("./loader-emit!");
		require("./loader-emit-hide!");
	});
});

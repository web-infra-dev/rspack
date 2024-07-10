it("should hide stack in details when throw", function () {
	expect(function f() {
		require("./loader-throw!");
		require("./loader-throw-hide!");
	}).toThrowError();
});

it("should hide stack in details when emit", function () {
	expect(function f() {
		require("./loader-emit!");
		require("./loader-emit-hide!");
	});
});

it("should not include a module with a weak dependency", function () {
	var a = !!__webpack_modules__[require.resolveWeak("./a")];
	var b = !!__webpack_modules__[require.resolve("./b")];
	var c = !!__webpack_modules__[require.resolveWeak("./c")];
	var d = !!__webpack_modules__[require.resolveWeak("./d")];
	import("./c");
	require("./d");

	if (require.resolveWeak && require.resolve) {
		expect(a).toBe(false);
		expect(b).toBe(true);
		expect(c).toBe(false);
		expect(d).toBe(true);
	} else {
		throw new Error(
			`'require.resolveWeak && require.resolve' should evaluate to truthy in IfStmt::Test`
		);
	}
});

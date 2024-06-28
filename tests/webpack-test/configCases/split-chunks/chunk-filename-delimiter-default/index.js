it("should run", function () {
	const files = require("fs").readdirSync(__dirname);
	expect(files).toContain("a.bundle.js");
	// CHANGE: RSPACK and WEBPACK has difference in hash
	// expect(files).toContain("b-b_js-c441f481.bundle.js");
	expect(
		files.some(file => /^b-b_js-[a-zA-Z\d]+\.bundle\.js$/.test(file))
	).toBeTruthy();

	return Promise.all([
		import(/* webpackChunkName: "a" */ "./a"),
		import(/* webpackChunkName: "b" */ "./b")
	]);
});

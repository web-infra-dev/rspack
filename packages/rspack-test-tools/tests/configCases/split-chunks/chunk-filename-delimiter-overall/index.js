it("should have files with overall delimiter('~')", function () {
	const files = require("fs").readdirSync(__dirname);
	expect(files).toContain("a.bundle.js");
	expect(files).toContain("async-commons_js.bundle.js");
	expect(
		files.some(file => /^b~b_js_[a-zA-Z\d]+\.bundle\.js$/.test(file))
	).toBeTruthy();
	expect(
		files.some(file => /^b~c_js_[a-zA-Z\d]+\.bundle\.js$/.test(file))
	).toBeTruthy();

	return Promise.all([
		import(/* webpackChunkName: "a" */ "./a"),
		import(/* webpackChunkName: "b" */ "./b")
	]);
});

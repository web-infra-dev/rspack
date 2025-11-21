
/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
	description: "should compile a single file",
	options(context) {
		return {
			entry: "./c",
		};
	},

	async check({ files }) {
		expect(Object.keys(files)).toEqual(["/main.js"]);
		const bundle = files["/main.js"];
		expect(bundle).toMatch("function __webpack_require__(");
		expect(bundle).toMatch("./c.js");
		expect(bundle).toMatch("./a.js");
		expect(bundle).toMatch("This is a");
		expect(bundle).toMatch("This is c");
		expect(bundle).not.toMatch("2: function(");
		expect(bundle).not.toMatch("window");
		expect(bundle).not.toMatch("jsonp");
		expect(bundle).not.toMatch("fixtures");
	}
}, {
	description: "should compile a complex file",
	options(context) {
		return {
			entry: "./main1",
		};
	},

	async check({ files }) {
		expect(Object.keys(files)).toEqual(["/main.js"]);
		const bundle = files["/main.js"];
		expect(bundle).toMatch("function __webpack_require__(");
		expect(bundle).toMatch("__webpack_require__(/*! ./a */");
		expect(bundle).toMatch("./main1.js");
		expect(bundle).toMatch("./a.js");
		expect(bundle).toMatch("./b.js");
		expect(bundle).toMatch("./node_modules/m1/a.js");
		expect(bundle).toMatch("This is a");
		expect(bundle).toMatch("This is b");
		expect(bundle).toMatch("This is m1/a");
		expect(bundle).not.toMatch("4: function(");
		expect(bundle).not.toMatch("window");
		expect(bundle).not.toMatch("jsonp");
		expect(bundle).not.toMatch("fixtures");
	}
}];

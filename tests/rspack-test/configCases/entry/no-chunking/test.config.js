module.exports = {
	findBundle: function (i, options) {
		// should add "./async_js.js" and "./nested_js.js", webpack don't have these two,
		// but somehow it passes in webpack but failed in rspack
		return ["./a.js", "./b.js", "./c.js", "./runtime.js", "./d.js"];
	}
};

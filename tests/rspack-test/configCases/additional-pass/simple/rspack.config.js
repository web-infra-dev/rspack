/** @type {import("@rspack/core").WebpackPluginFunction} */
var testPlugin = function () {
	var counter = 1;
	this.hooks.compilation.tap("TestPlugin", compilation => {
		var nr = counter++;
		compilation.hooks.needAdditionalPass.tap("TestPlugin", function () {
			if (nr < 5) return true;
		});
	});
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [testPlugin]
};

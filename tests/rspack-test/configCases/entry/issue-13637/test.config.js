const System = require("@rspack/test-tools/helper/legacy/fakeSystem");

module.exports = {
	beforeExecute: () => {
		System.init();
	},
	moduleScope(scope) {
		scope.System = System;
	},
	afterExecute: () => {
		System.execute("(anonym)");
	},
	findBundle: function () {
		return ["./main.system.js", "./main.umd.js"];
	}
};

const System = require("@rspack/test-tools/helper/legacy/fakeSystem");
module.exports = {
	beforeExecute: () => {
		System.init();
	},
	moduleScope(scope) {
		scope.System = System;
	},
	afterExecute: () => {
		System.execute(`named-system-module-main`);
	}
};

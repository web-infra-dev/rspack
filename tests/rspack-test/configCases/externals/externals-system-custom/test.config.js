const System = require("@rspack/test-tools/helper/legacy/fakeSystem");

module.exports = {
	target: 'web',
	beforeExecute: () => {
		System.init();
	},
	moduleScope(scope) {
		scope.window.windowExt = 'works';
		scope.rootExt = 'works';
		scope.varExt = 'works';
		scope.System = System;
	},
	afterExecute: () => {
		System.execute("(anonym)");
	}
};

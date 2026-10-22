import { System } from "@rspack/test-tools/helper/legacy/fakeSystem";

export default {
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

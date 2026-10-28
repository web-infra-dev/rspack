import { System } from "@rspack/test-tools/helper/legacy/fakeSystem";

export default {
	beforeExecute: () => {
		System.init();
	},
	moduleScope(scope) {
		scope.System = System;
	},
	afterExecute: () => {
		System.execute("(anonym)");
	}
};

import { createSystem } from "@rspack/test-tools/helper/legacy/fakeSystem";

const System = createSystem();

export default {
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

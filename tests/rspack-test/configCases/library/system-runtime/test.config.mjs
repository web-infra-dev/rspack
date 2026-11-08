import { createSystem } from "@rspack/test-tools/helper/legacy/fakeSystem";

const System = createSystem();

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	beforeExecute: () => {
		System.init();
	},
	findBundle() {
		return ["./main.js"];
	},
	moduleScope(scope) {
		System.setRequire(scope.require);
		scope.System = System;
	},
	afterExecute: () => {
		System.execute("(anonym)");
	}
};

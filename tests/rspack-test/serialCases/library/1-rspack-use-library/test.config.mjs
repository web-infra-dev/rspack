/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	moduleScope(scope) {
		scope.define = factory => {
			scope.module.exports = factory();
		};
	},
	afterExecute() {
		delete global.rspackChunk;
	}
};

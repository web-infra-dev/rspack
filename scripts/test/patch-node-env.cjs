// Copied from <https://github.com/webpack/webpack/blob/main/test/patch-node-env.js>
//
const NodeEnvironment =
	// For jest@29
	require("jest-environment-node").TestEnvironment ||
	// For jest@27
	require("jest-environment-node");

class CustomEnvironment extends NodeEnvironment {
	// Workaround for `Symbol('JEST_STATE_SYMBOL')`
	async handleTestEvent(event, state) {
		if (!this.global.JEST_STATE_SYMBOL) {
			this.global.JEST_STATE_SYMBOL = state;
		}
	}
}

module.exports = CustomEnvironment;

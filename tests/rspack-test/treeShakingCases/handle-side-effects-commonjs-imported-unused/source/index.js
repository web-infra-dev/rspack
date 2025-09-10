class Response {
	/**
	 * @param {CompiledMode} mode
	 */
	constructor(mode) {
		// eslint-disable-next-line no-undefined
		if (mode.data === undefined) mode.data = {};

		this.data = mode.data;
		this.isMatchIgnored = false;
	}

	ignoreMatch() {
		this.isMatchIgnored = true;
	}
}
function test() {
	let res = new Response();
	return res;
}
const result = test();

module.exports = result;

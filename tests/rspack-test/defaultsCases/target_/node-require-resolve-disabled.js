/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "target node with requireResolve disabled",
	options: () => ({
		target: "node",
		module: {
			parser: {
				javascript: {
					requireResolve: false
				}
			}
		}
	}),
	diff: e => {
		e.toEqual({
			value: expect.stringMatching(/\+\s+"requireResolve": false/)
		});
		e.toEqual({
			value: expect.stringMatching(/\+\s+"createRequire": true/)
		});
	}
};

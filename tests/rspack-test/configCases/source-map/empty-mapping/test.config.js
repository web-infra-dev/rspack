/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	checkCache({ namespace, cold, restored }) {
		if (namespace !== "|rspack.SourceMapDevToolPlugin") return;

		// The empty CSS has no source map, so only the JS asset is cached.
		expect(cold).toEqual({ hits: 0, misses: 2, reused: 0, stores: 1 });
		expect(restored).toEqual({ hits: 1, misses: 1, reused: 1, stores: 0 });
		return true;
	}
};

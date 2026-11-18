/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	checkCache({ namespace, cold, restored }) {
		if (namespace !== "|Compilation/modules") return;

		// The loader hook marks value.txt non-cacheable; only index.js is reused.
		expect(cold).toEqual({ hits: 0, misses: 2, reused: 0, stores: 2 });
		expect(restored).toEqual({ hits: 2, misses: 0, reused: 1, stores: 1 });
		return true;
	}
};

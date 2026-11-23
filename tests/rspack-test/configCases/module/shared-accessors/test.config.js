/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	checkCache({ namespace, cold, restored }) {
		if (namespace !== "|Compilation/modules") return;

		// Both modules must rerun the non-cacheable loader to exercise its setters.
		expect(cold).toEqual({ hits: 0, misses: 2, reused: 0, stores: 2 });
		expect(restored).toEqual({ hits: 2, misses: 0, reused: 0, stores: 2 });
		return true;
	}
};

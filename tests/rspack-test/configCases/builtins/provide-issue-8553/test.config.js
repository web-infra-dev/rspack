/** @type {import('@rspack/test-tools').TConfigCaseConfig} */
module.exports = {
	checkCache({ namespace, cold, restored }) {
		if (namespace !== "|Compilation/modules") return;

		// worker-rspack-loader disables caching for its parent module in pitch().
		// Child compiler namespaces retain the default full-reuse check.
		expect(cold).toEqual({ hits: 0, misses: 3, reused: 0, stores: 3 });
		expect(restored).toEqual({ hits: 3, misses: 0, reused: 2, stores: 1 });
		return true;
	}
};

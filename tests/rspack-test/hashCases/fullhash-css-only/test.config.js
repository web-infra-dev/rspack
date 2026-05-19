/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
	validate(stats) {
		expect(stats.stats).toHaveLength(4);

		const builtin0 = stats.stats[0].toJson({ assets: true });
		const builtin1 = stats.stats[1].toJson({ assets: true });
		const extract0 = stats.stats[2].toJson({ assets: true });
		const extract1 = stats.stats[3].toJson({ assets: true });

		const assertHashes = (before, after) => {
			const jsBefore = before.assets.find(asset => asset.name.endsWith(".js"));
			const jsAfter = after.assets.find(asset => asset.name.endsWith(".js"));
			const cssBefore = before.assets.find(asset => asset.name.endsWith(".css"));
			const cssAfter = after.assets.find(asset => asset.name.endsWith(".css"));

			expect(jsBefore).toBeDefined();
			expect(jsAfter).toBeDefined();
			expect(cssBefore).toBeDefined();
			expect(cssAfter).toBeDefined();

			expect(jsBefore.name).toBe(`main.${before.hash}.js`);
			expect(jsAfter.name).toBe(`main.${after.hash}.js`);
			expect(cssBefore.name).not.toBe(cssAfter.name);
			expect(before.hash).not.toBe(after.hash);
		};

		assertHashes(builtin0, builtin1);
		assertHashes(extract0, extract1);
	}
};

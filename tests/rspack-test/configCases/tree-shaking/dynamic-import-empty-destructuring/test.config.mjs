export default {
	validate({ stats }) {
		const source = stats.flatMap(stat => stat.compilation.getAssets())
			.map(asset => asset.source.source().toString())
			.join("\n");
		for (const marker of [
			"EMPTY_LAZY_PURE_UNUSED",
			"EMPTY_LAZY_EFFECTFUL_UNUSED",
			"EMPTY_EAGER_UNUSED",
			"EMPTY_CONTEXT_PURE_UNUSED",
			"EMPTY_CONTEXT_EFFECTFUL_UNUSED",
			"EMPTY_DECLARED_UNUSED",
			"EMPTY_DECLARED_HELPER_UNUSED",
			"EMPTY_WEAK_UNUSED",
			"EMPTY_UNUSED_BINDING_UNUSED",
			"EMPTY_THEN_CALLBACK_UNUSED",
		]) {
			expect(source).not.toContain(marker);
		}
	},
};

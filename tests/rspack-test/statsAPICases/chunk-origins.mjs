/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
export default {
	description:
		"should skip unused chunk origins and retain raw origins for custom stats factories",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/chunk-b"
		};
	},
	check(stats) {
		const disabled = stats.toJson({
			all: false,
			chunks: true,
			chunkOrigins: false
		});
		expect(
			disabled.chunks.every((chunk) => !Object.hasOwn(chunk, "origins"))
		).toBe(true);

		const enabled = stats.toJson({
			all: false,
			chunks: true,
			chunkOrigins: true
		});
		expect(enabled.chunks.some((chunk) => chunk.origins?.length > 0)).toBe(
			true
		);

		let rawOriginCount = 0;
		stats.compilation.hooks.statsFactory.tap(
			"ChunkOriginsCompatibilityTest",
			(statsFactory) => {
				statsFactory.hooks.extract
					.for("chunk")
					.tap("ChunkOriginsCompatibilityTest", (_object, chunk) => {
						rawOriginCount = Math.max(
							rawOriginCount,
							chunk.origins?.length ?? 0
						);
					});
			}
		);

		const custom = stats.toJson({
			all: false,
			chunks: true,
			chunkOrigins: false
		});
		expect(
			custom.chunks.every((chunk) => !Object.hasOwn(chunk, "origins"))
		).toBe(true);
		expect(rawOriginCount).toBeGreaterThan(0);

		rawOriginCount = 0;
		stats.toString({ all: false, chunks: true, chunkOrigins: false });
		expect(rawOriginCount).toBeGreaterThan(0);
	}
};

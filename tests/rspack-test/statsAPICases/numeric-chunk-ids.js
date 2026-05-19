function expectChunkId(id) {
	expect(typeof id).toBe("number");
}

function expectChunkIdArray(ids) {
	expect(Array.isArray(ids)).toBe(true);
	expect(ids.length).toBeGreaterThan(0);
	for (const id of ids) {
		expectChunkId(id);
	}
}

function expectNullableChunkIdArray(ids) {
	expect(Array.isArray(ids)).toBe(true);
	expect(ids.length).toBeGreaterThan(0);
	for (const id of ids) {
		if (id !== null && id !== undefined) {
			expectChunkId(id);
		}
	}
}

function expectStatsChunkGroup(group) {
	expectChunkIdArray(group.chunks);
	for (const childGroup of Object.values(group.children || {})) {
		for (const child of childGroup) {
			expectStatsChunkGroup(child);
		}
	}
}

/** @type {import("@rspack/test-tools").TStatsAPICaseConfig} */
module.exports = {
	description: "should expose numeric chunk ids as numbers in stats JSON",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./fixtures/order/index"
			},
			output: {
				environment: {
					methodShorthand: false
				}
			},
			optimization: {
				chunkIds: "deterministic",
				minimize: false
			},
			devtool: "source-map"
		};
	},
	async check(stats) {
		const json = stats.toJson({
			all: false,
			assets: true,
			chunks: true,
			chunkRelations: true,
			modules: true,
			ids: true,
			entrypoints: true,
			chunkGroups: true,
			chunkGroupAuxiliary: true,
			chunkGroupChildren: true
		});

		expect(json.assets.length).toBeGreaterThan(0);
		expect(json.assets.some(asset => asset.chunks?.length)).toBe(true);
		for (const asset of json.assets) {
			if (asset.chunks?.length) {
				expectNullableChunkIdArray(asset.chunks);
			}
			if (asset.auxiliaryChunks?.length) {
				expectNullableChunkIdArray(asset.auxiliaryChunks);
			}
		}

		expect(json.modules.length).toBeGreaterThan(0);
		expect(json.modules.some(module => module.chunks?.length)).toBe(true);
		for (const module of json.modules) {
			if (module.chunks?.length) {
				expectChunkIdArray(module.chunks);
			}
		}

		expect(json.chunks.length).toBeGreaterThan(1);
		expect(json.chunks.some(chunk => chunk.children?.length)).toBe(true);
		expect(
			json.chunks.some(chunk =>
				Object.values(chunk.childrenByOrder || {}).some(children => children.length)
			)
		).toBe(true);
		for (const chunk of json.chunks) {
			expectChunkId(chunk.id);
			for (const relatedChunks of [
				chunk.parents,
				chunk.children,
				chunk.siblings
			]) {
				if (relatedChunks?.length) {
					expectChunkIdArray(relatedChunks);
				}
			}
			for (const children of Object.values(chunk.childrenByOrder || {})) {
				if (children.length) {
					expectChunkIdArray(children);
				}
			}
		}

		const entrypoints = Object.values(json.entrypoints || {});
		const namedChunkGroups = Object.values(json.namedChunkGroups || {});
		expect(entrypoints.length).toBeGreaterThan(0);
		expect(namedChunkGroups.length).toBeGreaterThan(0);
		for (const entrypoint of entrypoints) {
			expectStatsChunkGroup(entrypoint);
		}
		for (const chunkGroup of namedChunkGroups) {
			expectStatsChunkGroup(chunkGroup);
		}
	}
};

import { MultiStats } from "@rspack/core";

/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
export default {
	description: "should honor MultiStats version and boolean child options",
	options(context) {
		return { context: context.getSource(), entry: "./fixtures/a" };
	},
	check(stats) {
		expect(stats.toJson(false)).toEqual({});
		expect(stats.toString(false)).toBe("");
		expect(stats.toJson(true).assets.length).toBeGreaterThan(0);
		const multi = new MultiStats([stats, stats]);
		expect(multi.toJson(true).children[0].assets.length).toBeGreaterThan(0);
		const version = stats.toJson({ all: false, version: true });
		const enabled = multi.toJson({ all: false, version: true });
		expect(enabled.version).toBe(version.version);
		expect(enabled.rspackVersion).toBe(version.rspackVersion);

		const disabled = multi.toJson({ all: false, version: false });
		expect(disabled.version).toBeUndefined();
		expect(disabled.rspackVersion).toBeUndefined();

		const children = multi.toJson({
			all: false,
			version: true,
			assets: true,
			children: [true, false]
		});
		expect(children.version).toBeUndefined();
		expect(children.rspackVersion).toBeUndefined();
		expect(children.children[0].version).toBe(version.version);
		expect(children.children[0].assets.length).toBeGreaterThan(0);
		expect(children.children[1].version).toBeUndefined();
		expect(children.children[1].assets).toBeUndefined();

		const allChildren = multi.toJson({
			all: false,
			version: false,
			children: true
		});
		expect(allChildren.version).toBe(version.version);
		expect(allChildren.rspackVersion).toBe(version.rspackVersion);
		expect(allChildren.children[0].assets.length).toBeGreaterThan(0);
		expect(allChildren.children[1].version).toBe(version.version);

		const mixed = multi.toJson({
			all: false,
			version: true,
			children: ["none", { version: false }]
		});
		expect(mixed.version).toBeUndefined();
		expect(mixed.children[0].version).toBe(version.version);
		expect(mixed.children[1].version).toBeUndefined();
		expect(
			multi.toString({
				all: false,
				version: true,
				children: [false, { version: true }]
			})
		).toBe(stats.toString({ all: false, version: true }));
		expect(multi.toString(false)).toBe("");
	}
};

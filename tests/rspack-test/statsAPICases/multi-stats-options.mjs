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

		const multi = new MultiStats([stats, stats]);
		const version = stats.toJson({ all: false, version: true });
		expect(multi.toJson({ all: false, version: true }).version).toBe(
			version.version
		);
		expect(
			multi.toJson({ all: false, version: false }).version
		).toBeUndefined();

		const children = multi.toJson({
			all: false,
			version: true,
			children: [true, false]
		});
		expect(children.version).toBeUndefined();
		expect(children.children[0].version).toBe(version.version);
		expect(children.children[1]).toEqual({});
	}
};

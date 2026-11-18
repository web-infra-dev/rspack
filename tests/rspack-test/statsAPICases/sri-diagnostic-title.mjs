import { SubresourceIntegrityPlugin } from "@rspack/core";

/** @type {import("@rspack/test-tools").TStatsAPICaseConfig} */
export default {
	description: "should display the SRI plugin name in errors and warnings",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/a",
			mode: "development",
			plugins: [new SubresourceIntegrityPlugin({ enabled: true })]
		};
	},
	check(stats) {
		const options = { all: false, errors: true, warnings: true, colors: false };
		const output = stats.toString(options);
		expect(output).toContain(
			"ERROR in × SubresourceIntegrityPlugin: Subresource integrity is not applied"
		);
		expect(output).toContain(
			"WARNING in ⚠ SubresourceIntegrityPlugin: SRI requires a cross-origin policy"
		);
		expect(stats.toJson(options).errors[0].code).toBe("SubresourceIntegrity");
	}
};

import { CopyRspackPlugin } from "@rspack/core";

/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
export default {
	description: "should distinguish missing files from files excluded by ignore",
	options() {
		return {
			entry: "./file",
			plugins: [
				new CopyRspackPlugin({
					patterns: [
						{ from: "missing-*.txt", globOptions: { ignore: ["**/*"] } },
						{ from: "file.js", globOptions: { ignore: ["**/file.js"] } }
					]
				})
			]
		};
	},
	check({ errors }) {
		expect(errors).toHaveLength(2);
		expect(errors.map(error => error.message)).toEqual(expect.arrayContaining([
			expect.stringMatching(/No files matched '.*missing-\*\.txt'\. Check `patterns\[0\]\.from`\./),
			expect.stringContaining("`patterns[1].globOptions.ignore` excluded every file matching 'file.js' (1 total).")
		]));
	}
};

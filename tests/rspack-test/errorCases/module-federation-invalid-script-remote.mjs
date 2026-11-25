import { container } from "@rspack/core";

/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
export default {
	description: "should report the name and expected format of an invalid script remote",
	options() {
		return {
			entry: "./file",
			plugins: [
				new container.ModuleFederationPlugin({
					name: "host",
					remotes: { shop: "https://example.com/remoteEntry.js" }
				})
			]
		};
	},
	check({ errors }) {
		expect(errors.map(error => error.message)).toEqual([
			'[ModuleFederationPlugin] remotes["shop"] must use the format "<global>@<url>" for a script remote. Received "https://example.com/remoteEntry.js".'
		]);
	}
};

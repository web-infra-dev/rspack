const { ModuleFederationPlugin } = require("@rspack/core").container;

class AssertEncodedMFRuntimeDataUriPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap(
			"AssertEncodedMFRuntimeDataUriPlugin",
			(_, { normalModuleFactory }) => {
				normalModuleFactory.hooks.beforeResolve.tap(
					"AssertEncodedMFRuntimeDataUriPlugin",
					(resolveData) => {
						const request = resolveData?.request;
						const prefix =
							"@module-federation/runtime/rspack.js!=!data:text/javascript,";

						if (!request || !request.startsWith(prefix)) {
							return;
						}

						const dataUriContent = request.slice(prefix.length);

						expect(dataUriContent).toMatch(/%[0-9A-Fa-f]{2}/);
						expect(dataUriContent).not.toContain(
							"import __module_federation_bundler_runtime__ from",
						);
					},
				);
			},
		);
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new ModuleFederationPlugin({
			name: "container",
			filename: "container.js",
			library: { type: "commonjs-module" },
			exposes: ["./module"],
		}),
		new AssertEncodedMFRuntimeDataUriPlugin(),
	],
};

// Store import filter calls for verification using globalThis
globalThis.__importFilterCalls = [];

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	entry: {
		main: "./index.js"
	},
	module: {
		rules: [
			{
				test: /\.css/,
				type: "css/auto"
			}
		],
		parser: {
			"css/auto": {
				resolveImport: (ctx) => {
				const { url, media, resourcePath, supports, layer } = ctx || {};
				globalThis.__importFilterCalls.push({
					url,
					media,
					resourcePath,
					supports,
					layer
				});
				// Filter out d.css to verify filtering works
				const result = !url.includes("d.css");
				return result;
			}
			}
		},
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		}
	},
};

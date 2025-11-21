const rspack = require("@rspack/core");
const path = require("path");

const dllManifest = path.resolve(
	__dirname,
	"../../../js/config/dll/no-warning-for-cjs/manifest.json"
);

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		name: "create-dll",
		entry: "./lib.js",
		output: {
			filename: "lib-dll.js",
			library: {
				type: "commonjs2"
			}
		},
		plugins: [
			new rspack.DllPlugin({
				path: dllManifest,
				entryOnly: false
			})
		]
	},
	{
		name: "use-dll",
		dependencies: ["create-dll"],
		entry: "./main.js",
		plugins: [
			new rspack.DllReferencePlugin({
				manifest: dllManifest,
				sourceType: "commonjs2",
				scope: "dll",
				name: "./lib-dll.js"
			})
		]
	}
];

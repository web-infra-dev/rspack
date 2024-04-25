const { ContainerPlugin, ContainerReferencePlugin } = require("@rspack/core").container;

const RUNTIME = "container-runtime";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    filename: "[name].js"
  },
	plugins: [
		new ContainerPlugin({
			name: "A",
      runtime: RUNTIME,
      filename: "container-a.js",
			library: {
				type: "commonjs-module"
			},
			exposes: {
				".": "./a"
			}
		}),
		new ContainerPlugin({
			name: "B",
      runtime: RUNTIME,
      filename: "container-b.js",
			library: {
				type: "commonjs-module"
			},
			exposes: {
				".": "./b"
			}
		}),
    new ContainerReferencePlugin({
			remoteType: "commonjs-module",
      remotes: {
        "A": "./container-a.js",
        "B": "./container-b.js",
      }
    })
	]
};

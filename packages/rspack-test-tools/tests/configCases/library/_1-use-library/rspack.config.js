const { rspack } = require("@rspack/core");
var path = require("path");
/** @type {function(any, any): import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
	{
		resolve: {
			alias: {
				library: path.resolve(testPath, "../../0-create-library/dist/esm.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("esm")
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/esm-runtimeChunk/main.js"
				)
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("esm-runtimeChunk")
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs.js"
				)
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs")
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs-iife.js"
				)
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs-iife")
			})
		]
	},
	// {
	// 	resolve: {
	// 		alias: {
	// 			library: path.resolve(testPath, "../../0-create-library/dist/amd.js")
	// 		}
	// 	},
	// 	builtins: {
	// 		define: {
	// 			NAME: JSON.stringify("amd")
	// 		}
	// 	}
	// },
	// {
	// 	resolve: {
	// 		alias: {
	// 			library: path.resolve(testPath, "../../0-create-library/dist/amd-iife.js")
	// 		}
	// 	},
	// 	builtins: {
	// 		define: {
	// 			NAME: JSON.stringify("amd-iife")
	// 		}
	// 	}
	// },
	// {
	// 	externals: {
	// 		library: `promise (require(${JSON.stringify(
	// 			"../../0-create-library/dist/amd-runtimeChunk/runtime.js"
	// 		)}), require(${JSON.stringify(
	// 			"../../0-create-library/dist/amd-runtimeChunk/main.js"
	// 		)}))`
	// 	},
	// 	output: {
	// 		library: { type: "commonjs-module" }
	// 	},
	// 	builtins: {
	// 		define: {
	// 			NAME: JSON.stringify("amd-runtimeChunk")
	// 		}
	// 	}
	// },
	// {
	// 	externals: {
	// 		library: `promise (require(${JSON.stringify(
	// 			"../../0-create-library/dist/amd-iife-runtimeChunk/runtime.js"
	// 		)}), require(${JSON.stringify(
	// 			"../../0-create-library/dist/amd-iife-runtimeChunk/main.js"
	// 		)}))`
	// 	},
	// 	output: {
	// 		library: { type: "commonjs-module" }
	// 	},
	// 	builtins: {
	// 		define: {
	// 			NAME: JSON.stringify("amd-iife-runtimeChunk")
	// 		}
	// 	}
	// },
	{
		resolve: {
			alias: {
				library: path.resolve(testPath, "../../0-create-library/dist/umd.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("umd")
			})
		]
	},
	{
		entry: "./this-test.js",
		resolve: {
			alias: {
				library: path.resolve(testPath, "../../0-create-library/dist/this.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("this")
			})
		]
	},
	{
		entry: "./this-test.js",
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/this-iife.js"
				)
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("this-iife")
			})
		]
	},
	{
		entry: "./var-test.js",
		resolve: {
			alias: {
				library: path.resolve(testPath, "../../0-create-library/dist/var.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("var")
			})
		]
	},
	{
		entry: "./var-test.js",
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/var-iife.js"
				)
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("var-iife")
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs-nested.js"
				)
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs-nested")
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs-nested-iife.js"
				)
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs-nested-iife")
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs2-external.js"
				),
				external: path.resolve(__dirname, "node_modules/external.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs2 with external"),
				TEST_EXTERNAL: true
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs2-iife-external.js"
				),
				external: path.resolve(__dirname, "node_modules/external.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs2-iife with external"),
				TEST_EXTERNAL: true
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs2-external-eval.js"
				),
				external: path.resolve(__dirname, "node_modules/external.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs2 with external and eval devtool"),
				TEST_EXTERNAL: true
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs2-external-eval-source-map.js"
				),
				external: path.resolve(__dirname, "node_modules/external.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify(
					"commonjs2 with external and eval-source-map devtool"
				),
				TEST_EXTERNAL: true
			})
		]
	},
	{
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs-static-external.js"
				),
				external: path.resolve(__dirname, "node_modules/external.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs-static with external"),
				TEST_EXTERNAL: true
			})
		]
	},
	// __nested_webpack_require_
	// {
	// 	resolve: {
	// 		alias: {
	// 			library: path.resolve(
	// 				testPath,
	// 				"../../0-create-library/dist/commonjs2-split-chunks/main.js"
	// 			),
	// 			external: path.resolve(__dirname, "node_modules/external.js")
	// 		}
	// 	},
	// 	builtins: {
	// 		define: {
	// 			NAME: JSON.stringify("commonjs2 with splitChunks")
	// 		}
	// 	}
	// },
	// CI crash
	// {
	// 	entry: "./default-test.js",
	// 	resolve: {
	// 		alias: {
	// 			library: path.resolve(
	// 				testPath,
	// 				"../../0-create-library/dist/umd-default.js"
	// 			)
	// 		}
	// 	},
	// 	builtins: {
	// 		define: {
	// 			NAME: JSON.stringify("default")
	// 		}
	// 	}
	// },
	{
		externals: {
			library: `promise require(${JSON.stringify(
				path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs2-runtimeChunk/main.js"
				)
			)})`
		},
		output: {
			library: { type: "commonjs-module" }
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs2-runtimeChunk")
			})
		]
	},
	{
		externals: {
			library: `promise require(${JSON.stringify(
				path.resolve(
					testPath,
					"../../0-create-library/dist/commonjs2-iife-runtimeChunk/main.js"
				)
			)})`
		},
		output: {
			library: { type: "commonjs-module" }
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("commonjs2-iife-runtimeChunk")
			})
		]
	},
	// CI crash
	// {
	// 	externals: {
	// 		library: `var (require(${JSON.stringify(
	// 			"../../0-create-library/dist/global-runtimeChunk/runtime.js"
	// 		)}), require(${JSON.stringify(
	// 			"../../0-create-library/dist/global-runtimeChunk/main.js"
	// 		)}), globalName.x.y)`
	// 	},
	// 	target: "web",
	// 	builtins: {
	// 		define: {
	// 			NAME: JSON.stringify("global-runtimeChunk")
	// 		}
	// 	}
	// },
	// {
	// 	externals: {
	// 		library: `var (require(${JSON.stringify(
	// 			"../../0-create-library/dist/global-iife-runtimeChunk/runtime.js"
	// 		)}), require(${JSON.stringify(
	// 			"../../0-create-library/dist/global-iife-runtimeChunk/main.js"
	// 		)}), globalName.x.y)`
	// 	},
	// 	target: "web",
	// 	builtins: {
	// 		define: {
	// 			NAME: JSON.stringify("global-iife-runtimeChunk")
	// 		}
	// 	}
	// },

	{
		resolve: {
			alias: {
				library: path.resolve(testPath, "../../0-create-library/dist/entryA.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("entryA")
			})
		]
	}
	// {
	// 	resolve: {
	// 		alias: {
	// 			library: path.resolve(testPath, "../../0-create-library/dist/entryB.js")
	// 		}
	// 	},
	// 	plugins: [
	// 		new webpack.DefinePlugin({
	// 			NAME: JSON.stringify("entryB")
	// 		})
	// 	]
	// },
	// {
	// 	resolve: {
	// 		alias: {
	// 			library: path.resolve(testPath, "../../0-create-library/dist/entryC.js")
	// 		}
	// 	},
	// 	plugins: [
	// 		new webpack.DefinePlugin({
	// 			NAME: JSON.stringify("entryC")
	// 		})
	// 	]
	// }
];

const path = require("path");
const { rspack } = require("@rspack/core");
/** @type {function(any, any): import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
	{
		output: {
			uniqueName: "esm",
			filename: "esm.js",
			library: { type: "module" },
			module: true
		},
		target: "node14",
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
	},
	{
		output: {
			uniqueName: "modern-module",
			filename: "modern-module.js",
			library: { type: "modern-module" },
			module: true
		},
		target: "node14",
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		optimization: {
			avoidEntryIife: true
		}
	},
	{
		output: {
			uniqueName: "esm-runtimeChunk",
			filename: "esm-runtimeChunk/[name].js",
			library: { type: "module" },
			module: true
		},
		target: "node14",
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		optimization: {
			runtimeChunk: "single"
		},
	},
	{
		output: {
			uniqueName: "commonjs",
			filename: "commonjs.js",
			library: { type: "commonjs" },
			iife: false
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "commonjs-iife",
			filename: "commonjs-iife.js",
			library: { type: "commonjs" },
			iife: true
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "amd",
			filename: "amd.js",
			library: { type: "amd" },
			iife: false
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "amd-iife",
			filename: "amd-iife.js",
			library: { type: "amd" },
			iife: true
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "amd-runtimeChunk",
			filename: "amd-runtimeChunk/[name].js",
			library: { type: "amd" },
			globalObject: "global",
			iife: false
		},
		target: "web",
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		optimization: {
			runtimeChunk: "single"
		}
	},
	{
		output: {
			uniqueName: "amd-iife-runtimeChunk",
			filename: "amd-iife-runtimeChunk/[name].js",
			library: { type: "amd" },
			globalObject: "global",
			iife: true
		},
		target: "web",
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		optimization: {
			runtimeChunk: "single"
		}
	},
	{
		output: {
			uniqueName: "umd",
			filename: "umd.js",
			library: { type: "umd" }
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "true-iife-umd",
			filename: "true-iife-umd.js",
			library: {
				type: "umd"
			},
			iife: true
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "false-iife-umd",
			filename: "false-iife-umd.js",
			library: {
				type: "umd"
			},
			iife: false
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		ignoreWarnings: [error => error.name === "FalseIIFEUmdWarning"]
	},
	{
		output: {
			uniqueName: "false-iife-umd2",
			filename: "false-iife-umd2.js",
			library: {
				type: "umd2"
			},
			iife: false
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		ignoreWarnings: [error => error.name === "FalseIIFEUmdWarning"]
	},
	{
		output: {
			uniqueName: "umd-default",
			filename: "umd-default.js",
			library: { type: "umd", export: "default" },
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "this",
			filename: "this.js",
			library: { type: "this" },
			iife: false
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "this-iife",
			filename: "this-iife.js",
			library: { type: "this" },
			iife: true
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "var",
			filename: "var.js",
			library: ["globalName", "x", "y"],
			iife: false
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		plugins: [
			new rspack.BannerPlugin({
				raw: true,
				banner: "module.exports = () => globalName;\n"
			})
		]
	},
	{
		output: {
			uniqueName: "var-iife",
			filename: "var-iife.js",
			library: ["globalName", "x", "y"],
			iife: true
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		plugins: [
			new rspack.BannerPlugin({
				raw: true,
				banner: "module.exports = () => globalName;\n"
			})
		]
	},
	{
		entry: "./nested.js",
		output: {
			uniqueName: "commonjs-nested",
			filename: "commonjs-nested.js",
			library: { type: "commonjs", export: "NS" },
			iife: false
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		entry: "./nested.js",
		output: {
			uniqueName: "commonjs-nested-iife",
			filename: "commonjs-nested-iife.js",
			library: { type: "commonjs", export: "NS" },
			iife: true
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "commonjs2-external",
			filename: "commonjs2-external.js",
			library: { type: "commonjs2" },
			iife: false
		},
		externals: ["external", "external-named"]
	},
	{
		output: {
			uniqueName: "commonjs2-external-no-concat",
			filename: "commonjs2-external-no-concat.js",
			library: { type: "commonjs2" },
			iife: false
		},
		optimization: {
			concatenateModules: false
		},
		externals: ["external", "external-named"]
	},
	{
		output: {
			uniqueName: "commonjs2-iife-external",
			filename: "commonjs2-iife-external.js",
			library: { type: "commonjs2" },
			iife: true
		},
		externals: ["external", "external-named"]
	},
	{
		mode: "development",
		output: {
			uniqueName: "commonjs2-external-eval",
			filename: "commonjs2-external-eval.js",
			library: { type: "commonjs2" }
		},
		externals: ["external", "external-named"]
	},
	{
		mode: "development",
		output: {
			uniqueName: "commonjs2-external-eval-source-map",
			filename: "commonjs2-external-eval-source-map.js",
			library: { type: "commonjs2" }
		},
		devtool: "eval-source-map",
		externals: ["external", "external-named"]
	},
	{
		output: {
			uniqueName: "commonjs-static-external",
			filename: "commonjs-static-external.js",
			library: { type: "commonjs-static" },
			iife: false
		},
		externals: ["external", "external-named"]
	},
	{
		output: {
			uniqueName: "index",
			filename: "index.js",
			path: path.resolve(testPath, "commonjs2-split-chunks"),
			library: { type: "commonjs2" }
		},
		target: "node",
		optimization: {
			splitChunks: {
				cacheGroups: {
					test: {
						enforce: true,
						chunks: "all",
						test: /a\.js$/,
						filename: "part.js"
					}
				}
			}
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	},
	{
		output: {
			uniqueName: "commonjs2-runtimeChunk",
			filename: "commonjs2-runtimeChunk/[name].js",
			library: { type: "commonjs2" },
			iife: false
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		optimization: {
			runtimeChunk: "single"
		}
	},
	{
		output: {
			uniqueName: "commonjs2-iife-runtimeChunk",
			filename: "commonjs2-iife-runtimeChunk/[name].js",
			library: { type: "commonjs2" },
			iife: true
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		optimization: {
			runtimeChunk: "single"
		}
	},
	{
		output: {
			uniqueName: "global-runtimeChunk",
			filename: "global-runtimeChunk/[name].js",
			library: { type: "global", name: ["globalName", "x", "y"] },
			iife: false
		},
		target: "web",
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		optimization: {
			runtimeChunk: "single"
		}
	},
	{
		output: {
			uniqueName: "global-iife-runtimeChunk",
			filename: "global-iife-runtimeChunk/[name].js",
			library: { type: "global", name: ["globalName", "x", "y"] },
			iife: true
		},
		target: "web",
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		},
		optimization: {
			runtimeChunk: "single"
		}
	},
	{
		entry: {
			entryA: {
				import: "./index"
			},
			entryB: {
				import: "./index",
				library: {
					type: "umd",
					name: "umd"
				}
			},
			entryC: {
				import: "./index",
				library: {
					type: "amd"
				}
			}
		},
		output: {
			library: {
				type: "commonjs-module"
			},
			uniqueName: "commonjs-module",
			filename: "[name].js"
		},
		resolve: {
			alias: {
				external: "./non-external",
				"external-named": "./non-external-named"
			}
		}
	}
];

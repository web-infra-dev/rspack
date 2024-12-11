import { type LibConfig, defineConfig } from "@rslib/core";
import prebundleConfig from "./prebundle.config.mjs";

const externalFunction = ({ request }: { request?: string }, callback) => {
	const { dependencies } = prebundleConfig;

	for (const item of dependencies) {
		const depName = typeof item === "string" ? item : item.name;
		if (new RegExp(`^${depName}$`).test(request!)) {
			return callback(null, `../compiled/${depName}/index.js`);
		}
	}

	if (/..\/package\.json/.test(request!)) {
		return callback(null, "../package.json");
	}

	return callback();
};

const commonLibConfig: LibConfig = {
	dts: false,
	format: "cjs",
	syntax: ["node 16"],
	source: {
		define: {
			__webpack_require__: "__webpack_require__"
		}
	},
	output: {
		cleanDistPath: false,
		// distPath: {
		// 	root: "./dist-rslib"
		// },
		externals: [externalFunction]
	}
};

export default defineConfig({
	lib: [
		{
			...commonLibConfig,
			source: {
				entry: {
					index: "./src/index.ts"
				}
			},
			output: {
				...commonLibConfig.output,
				externals: [externalFunction, "./moduleFederationDefaultRuntime.js"]
			},
			footer: {
				js: `
				module.exports = rspack;
				0 && (module.exports = {
  BannerPlugin,
  Compilation,
  Compiler,
  ContextReplacementPlugin,
  CopyRspackPlugin,
  CssExtractRspackPlugin,
  DefinePlugin,
  DllPlugin,
  DllReferencePlugin,
  DynamicEntryPlugin,
  EntryOptionPlugin,
  EntryPlugin,
  EnvironmentPlugin,
  EvalDevToolModulePlugin,
  EvalSourceMapDevToolPlugin,
  ExternalsPlugin,
  HotModuleReplacementPlugin,
  HtmlRspackPlugin,
  IgnorePlugin,
  LightningCssMinimizerRspackPlugin,
  LoaderOptionsPlugin,
  LoaderTargetPlugin,
  ModuleFilenameHelpers,
  MultiCompiler,
  MultiStats,
  NoEmitOnErrorsPlugin,
  NormalModule,
  NormalModuleReplacementPlugin,
  ProgressPlugin,
  ProvidePlugin,
  RspackOptionsApply,
  RuntimeGlobals,
  RuntimeModule,
  SourceMapDevToolPlugin,
  Stats,
  SwcJsMinimizerRspackPlugin,
  Template,
  ValidationError,
  WebpackError,
  WebpackOptionsApply,
  config,
  container,
  electron,
  experiments,
  javascript,
  library,
  node,
  optimize,
  rspack,
  rspackVersion,
  sharing,
  sources,
  util,
  version,
  wasm,
  web,
  webworker
});
`
			}
		},
		{
			...commonLibConfig,
			source: {
				entry: {
					cssExtractLoader: "./src/builtin-plugin/css-extract/loader.ts"
				}
			},
			footer: {
				js: `0 && (module.exports = {
  ABSOLUTE_PUBLIC_PATH,
  AUTO_PUBLIC_PATH,
  BASE_URI,
  MODULE_TYPE,
  SINGLE_DOT_PATH_SEGMENT,
  hotLoader,
  pitch
});`
			}
		},
		{
			...commonLibConfig,
			syntax: "es2015",
			source: {
				entry: {
					cssExtractHmr: "./src/runtime/cssExtractHmr.ts"
				}
			},
			footer: {
				js: `0 && (module.exports = {
  cssReload,
  normalizeUrl
});`
			}
		}
	],
	output: {
		target: "node"
	}
});

import type { Compiler, RspackPluginInstance } from "@rspack/core";
import fs from "fs";

export interface IPreactRefreshRspackPluginOptions {
	overlay?: {
		module: string;
	};
}

const PREACT_PATHS = [
	"preact",
	"preact/compat",
	"preact/debug",
	"preact/devtools",
	"preact/hooks",
	"preact/test-utils",
	"preact/jsx-runtime",
	"preact/jsx-dev-runtime",
	"preact/compat/client",
	"preact/compat/server",
	"preact/compat/jsx-runtime",
	"preact/compat/jsx-dev-runtime",
	"preact/compat/scheduler",
	"preact/package.json",
	"preact/compat/package.json",
	"preact/debug/package.json",
	"preact/devtools/package.json",
	"preact/hooks/package.json",
	"preact/test-utils/package.json",
	"preact/jsx-runtime/package.json"
].reduce((obj, i) => {
	obj[i] = require.resolve(i);
	return obj;
}, {} as Record<string, string>);
const PREFRESH_CORE_PATH = require.resolve("@prefresh/core");
const PREFRESH_UTILS_PATH = require.resolve("@prefresh/utils");
const RUNTIME_UTIL_PATH = require.resolve("../client/prefresh");
const RUNTIME_INTERCEPT_PATH = require.resolve("../client/intercept");

const INTERNAL_PATHS = [
	...Object.values(PREACT_PATHS),
	PREFRESH_UTILS_PATH,
	PREFRESH_CORE_PATH,
	PREFRESH_UTILS_PATH,
	RUNTIME_UTIL_PATH,
	RUNTIME_INTERCEPT_PATH
];

const runtimeSource = fs.readFileSync(RUNTIME_INTERCEPT_PATH, "utf-8");

const NAME = "PreactRefreshRsapckPlugin";

class PreactRefreshRsapckPlugin implements RspackPluginInstance {
	name = NAME;

	constructor(private options: IPreactRefreshRspackPluginOptions) {
		this.options = {
			overlay: options?.overlay
		};
	}

	apply(compiler: Compiler) {
		if (
			process.env.NODE_ENV === "production" ||
			compiler.options.mode === "production"
		)
			return;

		new compiler.webpack.ProvidePlugin({
			__prefresh_utils__: require.resolve("../client/prefresh"),
			...(this.options.overlay
				? {
						__prefresh_errors__: require.resolve(this.options.overlay.module)
				  }
				: {})
		}).apply(compiler);
		new compiler.webpack.EntryPlugin(compiler.context, "@prefresh/core", {
			name: undefined
		}).apply(compiler);
		// new compiler.webpack.DefinePlugin({ __refresh_library__ }).apply(compiler);
		compiler.options.resolve.alias = {
			"@prefresh/core": PREFRESH_CORE_PATH,
			"@prefresh/utils": PREFRESH_UTILS_PATH,
			...PREACT_PATHS,
			...compiler.options.resolve.alias
		};
		compiler.options.module.rules.unshift({
			include: /\.([jt]sx?)$/,
			exclude: {
				or: [/node_modules/, [...INTERNAL_PATHS]].filter(Boolean)
			},
			use: "builtin:preact-refresh-loader"
		});

		compiler.hooks.thisCompilation.tap(NAME, compilation => {
			compilation.hooks.runtimeModule.tap(NAME, (runtimeModule, chunk) => {
				// rspack does not have addRuntimeModule and runtimeRequirements on js side
				if (
					runtimeModule.constructorName === "HotModuleReplacementRuntimeModule"
				) {
					if (!runtimeModule.source) {
						throw new Error(
							"Can not get the original source of HotModuleReplacementRuntimeModule"
						);
					}
					const originalSource =
						runtimeModule.source.source.toString("utf-8") || "";
					runtimeModule.source.source = Buffer.from(
						`${originalSource}\n${runtimeSource}`,
						"utf-8"
					);
				}
			});
		});
	}
}

module.exports = PreactRefreshRsapckPlugin;

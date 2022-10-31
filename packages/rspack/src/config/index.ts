import type { Context, ResolvedContext } from "./context";
import type { Dev, ResolvedDev } from "./devServer";
import type { Entry, ResolvedEntry } from "./entry";
import { resolveEntryOptions } from "./entry";
import type {
	External,
	ExternalType,
	ResolvedExternal,
	ResolvedExternalType
} from "./external";
import type { Mode, ResolvedMode } from "./mode";
import type { Module, ResolvedModule, LoaderContext } from "./module";
import type { Plugin } from "./plugin";
import type { ResolvedTarget, Target } from "./target";
import type { Output, ResolvedOutput } from "./output";
import type { Resolve, ResolvedResolve } from "./resolve";
import type { Builtins, ResolvedBuiltins } from "./builtins";
import { Devtool, ResolvedDevtool, resolveDevtoolOptions } from "./devtool";
import { resolveTargetOptions } from "./target";
import { resolveOutputOptions } from "./output";
import { resolveDevOptions } from "./devServer";
import { resolveModuleOptions } from "./module";
import { resolveBuiltinsOptions } from "./builtins";
import { resolveResolveOptions } from "./resolve";
import { InfrastructureLogging } from "./RspackOptions";
import { Source } from "webpack-sources";

export interface RspackOptions {
	name?: string;
	entry?: Entry;
	context?: Context;
	plugins?: Plugin[];
	devServer?: Dev;
	module?: Module;
	target?: Target;
	mode?: Mode;
	externals?: External;
	externalsType?: ExternalType;
	output?: Output;
	builtins?: Builtins;
	resolve?: Resolve;
	devtool?: Devtool;
	infrastructureLogging?: InfrastructureLogging;
}
export interface RspackOptionsNormalized {
	name?: string;
	entry: ResolvedEntry;
	context: ResolvedContext;
	plugins: Plugin[];
	devServer: ResolvedDev;
	module: ResolvedModule;
	target: ResolvedTarget;
	mode: ResolvedMode;
	externals: ResolvedExternal;
	externalType: ResolvedExternalType;
	output: ResolvedOutput;
	builtins: ResolvedBuiltins;
	resolve: ResolvedResolve;
	devtool: ResolvedDevtool;
	infrastructureLogging: InfrastructureLogging;
}

export function getNormalizedRspackOptions(
	config: RspackOptions
): RspackOptionsNormalized {
	const context = config.context ?? process.cwd();
	const mode = config.mode ?? "production";
	const devServer = resolveDevOptions(config.devServer, { context });
	const entry = resolveEntryOptions(config.entry, {
		context,
		dev: !!config.devServer
	});
	const output = resolveOutputOptions(config.output);
	const target = resolveTargetOptions(config.target);
	const externals = config.externals ?? {};
	const externalType = config.externalsType ?? "";
	const plugins = config.plugins ?? [];
	const builtins = resolveBuiltinsOptions(config.builtins || {}, context);
	const resolve = resolveResolveOptions(config.resolve);
	const devtool = resolveDevtoolOptions(config.devtool);
	const module = resolveModuleOptions(config.module, { devtool, context });

	return {
		...config,
		context,
		mode,
		devServer,
		entry,
		output,
		target,
		externals,
		externalType,
		plugins,
		builtins,
		module,
		resolve,
		devtool,
		infrastructureLogging: cloneObject(config.infrastructureLogging)
	};
}

function cloneObject(value: Record<string, any> | undefined) {
	return { ...value };
}
export type { Plugin, LoaderContext };

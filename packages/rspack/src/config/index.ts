import type { Context, ResolvedContext } from "./context";
import type { Dev } from "./devServer";
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
import { resolveModuleOptions } from "./module";
import { resolveBuiltinsOptions } from "./builtins";
import { resolveResolveOptions } from "./resolve";
import { InfrastructureLogging } from "./RspackOptions";
import {
	ResolvedStatsOptions,
	resolveStatsOptions,
	StatsOptions
} from "./stats";

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
	stats?: StatsOptions;
}
export interface RspackOptionsNormalized {
	name?: string;
	entry: ResolvedEntry;
	context: ResolvedContext;
	plugins: Plugin[];
	devServer?: Dev;
	module: ResolvedModule;
	target: ResolvedTarget;
	mode: ResolvedMode;
	externals: ResolvedExternal;
	externalsType: ResolvedExternalType;
	output: ResolvedOutput;
	builtins: ResolvedBuiltins;
	resolve: ResolvedResolve;
	devtool: ResolvedDevtool;
	infrastructureLogging: InfrastructureLogging;
	stats: ResolvedStatsOptions;
}

export function getNormalizedRspackOptions(
	config: RspackOptions
): RspackOptionsNormalized {
	const context = config.context ?? process.cwd();
	const mode = config.mode ?? "production";
	const entry = resolveEntryOptions(config.entry, {
		context
	});
	const output = resolveOutputOptions(config.output);
	const target = resolveTargetOptions(config.target);
	const externals = config.externals ?? {};
	const externalsType = config.externalsType ?? "";
	const plugins = config.plugins ?? [];
	const builtins = resolveBuiltinsOptions(config.builtins || {}, context);
	const resolve = resolveResolveOptions(config.resolve);
	const devtool = resolveDevtoolOptions(config.devtool);
	const module = resolveModuleOptions(config.module, { devtool, context });
	const stats = resolveStatsOptions(config.stats);
	const devServer = config.devServer;

	return {
		...config,
		context,
		mode,
		devServer,
		entry,
		output,
		target,
		externals,
		externalsType,
		plugins,
		builtins,
		module,
		resolve,
		devtool,
		infrastructureLogging: cloneObject(config.infrastructureLogging),
		stats
	};
}

function cloneObject(value: Record<string, any> | undefined) {
	return { ...value };
}
export type { Plugin, LoaderContext };
export type { WebSocketServerOptions, Dev } from "./devServer";
export { resolveWatchOption } from "./watch";

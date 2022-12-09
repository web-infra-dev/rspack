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
import type {
	Module,
	ResolvedModule,
	LoaderContext,
	Loader,
	SourceMap
} from "./module";
import type { Plugin } from "./plugin";
import type { ResolvedTarget, Target } from "./target";
import type { Output, ResolvedOutput } from "./output";
import type { Resolve, ResolvedResolve } from "./resolve";
import type { Builtins, ResolvedBuiltins } from "./builtins";
import type { Snapshot, ResolvedSnapshot } from "./snapshot";
import type { Cache, ResolvedCache } from "./cache";
import { Devtool, ResolvedDevtool, resolveDevtoolOptions } from "./devtool";
import { resolveTargetOptions } from "./target";
import { resolveOutputOptions } from "./output";
import { resolveModuleOptions } from "./module";
import { resolveBuiltinsOptions } from "./builtins";
import { resolveResolveOptions } from "./resolve";
import { resolveSnapshotOptions } from "./snapshot";
import { resolveCacheOptions } from "./cache";
import { InfrastructureLogging } from "./RspackOptions";
import {
	ResolvedStatsOptions,
	resolveStatsOptions,
	StatsOptions
} from "./stats";
import {
	Optimization,
	ResolvedOptimization,
	resolveOptimizationOptions
} from "./optimization";

export type Configuration = RspackOptions;
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
	snapshot?: Snapshot;
	cache?: Cache;
	optimization?: Optimization;
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
	snapshot: ResolvedSnapshot;
	cache: ResolvedCache;
	optimization?: ResolvedOptimization;
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
	const builtins = resolveBuiltinsOptions(config.builtins || {}, {
		contextPath: context,
		isProduction: mode === "production"
	});
	const resolve = resolveResolveOptions(config.resolve, { target });
	const devtool = resolveDevtoolOptions(config.devtool);
	const module = resolveModuleOptions(config.module, { devtool, context });
	const stats = resolveStatsOptions(config.stats);
	const devServer = config.devServer;
	const snapshot = resolveSnapshotOptions(config.snapshot);
	const cache = resolveCacheOptions(
		config.cache ?? (mode === "production" ? false : true)
	);
	const optimization = resolveOptimizationOptions(
		config.optimization ?? {},
		mode
	);

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
		stats,
		snapshot,
		cache,
		optimization
	};
}

function cloneObject(value: Record<string, any> | undefined) {
	return { ...value };
}
export type { Plugin, LoaderContext, Loader, SourceMap };
export type { WebSocketServerOptions, Dev } from "./devServer";
export { resolveWatchOption } from "./watch";
export type { StatsOptions } from "./stats";

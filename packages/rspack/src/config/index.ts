import type { Context, ResolvedContext } from "./context";
import type { Define, ResolvedDefine } from "./define";
import type { Dev, ResolvedDev } from "./dev";
import type { Entry, ResolvedEntry } from "./entry";
import type { External, ResolvedExternal } from "./external";
import type { Mode, ResolvedMode } from "./mode";
import type { Module, ResolvedModule } from "./module";
import type { Plugin } from "./plugin";
import type { ResolvedTarget, Target } from "./target";
import type { Output, ResolvedOutput } from "./output";
import type { Resolve, ResolvedResolve } from "./resolve";
import type { Builtins, ResolvedBuiltins } from "./builtins";
import { resolveTargetOptions } from "./target";
import { resolveOutputOptions } from "./output";
import { resolveDevOptions } from "./dev";
import { resolveModuleOptions } from "./module";
import { resolveResolveOptions } from "./resolve";

export type Asset = {
	source: string;
};
export type Assets = Record<string, Asset>;

export interface RspackOptions {
	entry?: Entry;
	context?: Context;
	plugins?: Plugin[];
	dev?: Dev;
	module?: Module;
	define?: Define;
	target?: Target;
	mode?: Mode;
	external?: External;
	output?: Output;
	builtins?: Builtins;
	resolve?: Resolve;
}

export interface ResolvedRspackOptions {
	entry: ResolvedEntry;
	context: ResolvedContext;
	plugins: Plugin[];
	dev: ResolvedDev;
	module: ResolvedModule;
	define: ResolvedDefine;
	target: ResolvedTarget;
	mode: ResolvedMode;
	external: ResolvedExternal;
	output: ResolvedOutput;
	builtins: ResolvedBuiltins;
	resolve: ResolvedResolve;
}

export function resolveOptions(config: RspackOptions): ResolvedRspackOptions {
	const context = config.context ?? process.cwd();
	const mode = config.mode ?? "development";
	const dev = resolveDevOptions(config.dev, { context });
	const entry = config.entry ?? {};
	const output = resolveOutputOptions(config.output);
	const define = config.define ?? {};
	const target = resolveTargetOptions(config.target);
	const external = config.external ?? {};
	const plugins = config.plugins ?? [];
	const builtins = config.builtins ?? [];
	const resolve = resolveResolveOptions(config.resolve);
	const module = resolveModuleOptions(config.module);
	return {
		context,
		mode,
		dev,
		entry,
		output,
		define,
		target,
		external,
		plugins,
		builtins,
		module,
		resolve
	};
}

export type { Plugin };

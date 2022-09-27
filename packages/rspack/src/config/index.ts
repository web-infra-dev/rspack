import type { Context, ResolvedContext } from "./context";
import { Define, ResolvedDefine, resolveDefine } from "./define";
import type { Dev, ResolvedDev } from "./devServer";
import { Entry, ResolvedEntry, resolveEntryOptions } from "./entry";
import type {
	External,
	ExternalType,
	ResolvedExternal,
	ResolvedExternalType
} from "./external";
import type { Mode, ResolvedMode } from "./mode";
import type { Module, ResolvedModule } from "./module";
import type { Plugin } from "./plugin";
import type { ResolvedTarget, Target } from "./target";
import type { Output, ResolvedOutput } from "./output";
import type { Resolve, ResolvedResolve } from "./resolve";
import type { Builtins, ResolvedBuiltins } from "./builtins";
import type { Devtool, ResolvedDevtool } from "./devtool";
import { resolveTargetOptions } from "./target";
import { resolveOutputOptions } from "./output";
import { resolveDevOptions } from "./devServer";
import { resolveModuleOptions } from "./module";
import { resolveBuiltinsOptions } from "./builtins";
import { resolveResolveOptions } from "./resolve";
import { resolveEntry } from "./entry";

export type Asset = {
	source: string;
};
export type Assets = Record<string, Asset>;

export interface RspackOptions {
	entry?: Entry;
	context?: Context;
	plugins?: Plugin[];
	devServer?: Dev;
	module?: Module;
	define?: Define;
	target?: Target;
	mode?: Mode;
	externals?: External;
	externalsType?: ExternalType;
	output?: Output;
	builtins?: Builtins;
	resolve?: Resolve;
	devtool?: Devtool;
}

export interface ResolvedRspackOptions {
	entry: ResolvedEntry;
	context: ResolvedContext;
	plugins: Plugin[];
	devServer: ResolvedDev;
	module: ResolvedModule;
	define: ResolvedDefine;
	target: ResolvedTarget;
	mode: ResolvedMode;
	external: ResolvedExternal;
	externalType: ResolvedExternalType;
	output: ResolvedOutput;
	builtins: ResolvedBuiltins;
	resolve: ResolvedResolve;
	devtool: ResolvedDevtool;
}

export function resolveOptions(config: RspackOptions): ResolvedRspackOptions {
	const context = config.context ?? process.cwd();
	const mode = config.mode ?? "development";
	const devServer = resolveDevOptions(config.devServer, { context });
	const entry = resolveEntryOptions(config.entry ?? {}, {
		context,
		dev: !!config.devServer
	});
	const output = resolveOutputOptions(config.output);
	const define = resolveDefine(config.define);
	const target = resolveTargetOptions(config.target);
	const external = config.externals ?? {};
	const externalType = config.externalsType ?? "";
	const plugins = config.plugins ?? [];
	const builtins = resolveBuiltinsOptions(config.builtins || {}, context);
	const resolve = resolveResolveOptions(config.resolve);
	const module = resolveModuleOptions(config.module);
	const devtool = config.devtool ?? false;
	console.log("entry:", entry);
	return {
		context,
		mode,
		devServer,
		entry,
		output,
		define,
		target,
		external,
		externalType,
		plugins,
		builtins,
		module,
		resolve,
		devtool
	};
}

export type { Plugin };

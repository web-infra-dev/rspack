import type { Context, ResolvedContext } from "./context";
import type { Define, ResolvedDefine } from "./define";
import { Dev, ResolvedDev, resolveDevConfig } from "./dev";
import type { Entry, ResolvedEntry } from "./entry";
import type { External, ResolvedExternal } from "./external";
import type { Mode, ResolvedMode } from "./mode";
import type { Module, ResolvedModule } from "./module";
import type { Plugin } from "./plugin";
import type { ResolvedTarget, Target } from "./target";
import type { Output, ResolvedOutput } from "./output";
import { resolveOutput } from "./output";
import { createRawModuleRuleUses } from "./module";

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
}

export function resolveConfig(config: RspackOptions): ResolvedRspackOptions {
	return {
		mode: config.mode ?? "development",
		dev: resolveDevConfig(config.dev),
		entry: config.entry ?? {},
		context: config.context ?? process.cwd(),
		output: resolveOutput(config.output),
		define: config.define ?? {},
		target: config.target ?? "",
		external: config.external ?? {},
		plugins: config.plugins ?? [],
		module: {
			// TODO: support mutliple rules to support `Module Type`
			rules: (config?.module?.rules ?? []).map(rule => {
				return {
					...rule,
					uses: createRawModuleRuleUses(rule.uses || [])
				};
			})
		}
	};
}

import type { RawOptions } from "@rspack/binding";

import type { ModuleRule } from ".";
import { createRawModuleRuleUses } from ".";

export type Plugin = {
	name: string;
	done?: () => void | Promise<void>;
};

export interface RspackOptions {
	/**
	 * Entry points of compilation.
	 */
	entry?: RawOptions["entry"];
	/**
	 * An **absolute** path pointed the
	 */
	context?: RawOptions["context"];
	/**
	 * An array of plugins
	 */
	plugins?: Plugin[];
	/**
	 * dev server
	 */
	dev?: {
		port?: Number;
		static?: {
			directory?: string;
		};
	};
	/**
	 * Module configuration.
	 */
	module?: {
		rules?: ModuleRule[];
		parser?: RawOptions["module"]["parser"];
	};
	define?: RawOptions["define"];
	target?: RawOptions["target"];
	mode?: RawOptions["mode"];
	external?: RawOptions["external"];
	output?: RawOptions["output"];
}

export function User2Native(config: RspackOptions): RawOptions & {
	plugins: Plugin[];
} {
	return {
		entry: config.entry ?? {},
		context: config.context,
		output: config.output,
		define: config.define,
		target: config.target,
		external: config.external,
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

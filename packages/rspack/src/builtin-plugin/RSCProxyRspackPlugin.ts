import type { Compiler } from "../Compiler";
import type { RuleSetCondition } from "../config/zod";

interface Options {
	clientProxy: string;
	exclude?: RuleSetCondition;
}

export class RSCProxyRspackPlugin {
	options: Options;
	constructor(options: Options) {
		this.options = options;
	}
	apply(compiler: Compiler) {
		if (!compiler.options.module.rules) {
			compiler.options.module.rules = [];
		}
		compiler.options.module.rules.push({
			enforce: "post",
			test: [/\.(j|t|mj|cj)sx?$/i],
			exclude: this.options.exclude ?? {
				// Exclude libraries in node_modules ...
				and: [/node_modules/]
			},
			use: [
				{
					loader: "builtin:rsc-proxy-loader",
					options: this.options
				}
			]
		});
	}
}

import path from "node:path";
import { BuiltinPluginName } from "@rspack/binding";

import type { Compiler } from "../Compiler";
import type { RuleSetCondition } from "../config/zod";
import { create } from "./base";
import type { RspackBuiltinPlugin } from "./base";

const RawRSCProxyRspackPlugin = create(
	BuiltinPluginName.RSCProxyRspackPlugin,
	options => options,
	"compilation"
);

interface Options {
	clientProxy: string;
	exclude?: RuleSetCondition;
}

interface ResolvedOptions {
	root: string;
}

export class RSCProxyRspackPlugin {
	plugin: RspackBuiltinPlugin;
	options: Options;
	constructor(options: Options) {
		this.plugin = new RawRSCProxyRspackPlugin({});
		this.options = options;
	}
	apply(compiler: Compiler) {
		this.plugin.apply(compiler);
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
		const resolvedOptions = this.resolveOptions(compiler);
		compiler.options.module.rules.push({
			enforce: "pre",
			test: /rsc-server-action-entry-loader\.(j|t|mj|cj)sx?/,
			use: [
				{
					loader: "builtin:rsc-server-action-server-loader",
					options: resolvedOptions
				}
			]
		});
	}
	resolveOptions(compiler: Compiler): ResolvedOptions {
		const root = compiler.options.context ?? process.cwd();
		// TODO: config output
		const output = path.resolve(root, "./dist/server");
		return {
			root: output
		};
	}
}

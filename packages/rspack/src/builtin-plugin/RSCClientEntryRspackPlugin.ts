import { BuiltinPluginName } from "@rspack/binding";
import type { RawRscClientEntryRspackPluginOptions } from "@rspack/binding";

import path from "path";
import type { Compiler } from "../Compiler";
import { create } from "./base";
import type { RspackBuiltinPlugin } from "./base";

const RawRSCClientEntryRspackPlugin = create(
	BuiltinPluginName.RSCClientEntryRspackPlugin,
	options => options,
	"compilation"
);

interface Options extends RawRscClientEntryRspackPluginOptions {}

interface ResolvedOptions {
	root: string;
}

export class RSCClientEntryRspackPlugin {
	plugin: RspackBuiltinPlugin;
	options: Options;
	constructor(options: Options) {
		this.plugin = new RawRSCClientEntryRspackPlugin(options);
		this.options = options;
	}
	apply(compiler: Compiler) {
		this.plugin.apply(compiler);
		if (!compiler.options.module.rules) {
			compiler.options.module.rules = [];
		}
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

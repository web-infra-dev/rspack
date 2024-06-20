import path from "path";
import type { RawRscClientReferenceManifestRspackPluginOptions } from "@rspack/binding";
import { BuiltinPluginName } from "@rspack/binding";

import type { Compiler } from "../Compiler";
import type { RspackBuiltinPlugin } from "./base";
import { create } from "./base";

const RawRSCClientReferenceManifestRspackPlugin = create(
	BuiltinPluginName.RSCClientReferenceManifestRspackPlugin,
	options => options,
	"compilation"
);

interface ResolvedOptions {
	root: string;
}

interface Options extends RawRscClientReferenceManifestRspackPluginOptions {}

export class RSCClientReferenceManifestRspackPlugin {
	plugin: RspackBuiltinPlugin;
	options: Options;
	resolvedOptions: ResolvedOptions;
	constructor(options: Options) {
		this.plugin = new RawRSCClientReferenceManifestRspackPlugin({
			routes: options.routes
		});
		this.options = options;
		this.resolvedOptions = {} as any;
	}
	apply(compiler: Compiler) {
		this.plugin.apply(compiler);
		this.resolvedOptions = this.resolveOptions(compiler);
		if (!compiler.options.module.rules) {
			compiler.options.module.rules = [];
		}
		compiler.options.module.rules.push({
			test: /rsc-client-entry-loader\.(j|t|mj|cj)sx?/,
			use: [
				{
					loader: "builtin:rsc-client-entry-loader",
					options: this.resolvedOptions
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

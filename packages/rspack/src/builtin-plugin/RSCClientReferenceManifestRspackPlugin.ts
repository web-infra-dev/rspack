import type { RawRscClientEntryRspackPluginOptions } from "@rspack/binding";
import { BuiltinPluginName } from "@rspack/binding";
import path from "path";

import type { Compiler } from "../Compiler";
import type { RuleSetCondition } from "../config/zod";
import type { RspackBuiltinPlugin } from "./base";
import { create } from "./base";

const RawRSCClientReferenceManifestRspackPlugin = create(
	BuiltinPluginName.RSCClientReferenceManifestRspackPlugin,
	() => {},
	"compilation"
);

interface ResolvedOptions {
	routes: NonNullable<RawRscClientEntryRspackPluginOptions["routes"]>;
	entry: Record<string, string>;
	root: string;
}

interface Options extends Pick<RawRscClientEntryRspackPluginOptions, "routes"> {
	exclude?: RuleSetCondition;
}

export class RSCClientReferenceManifestRspackPlugin {
	plugin: RspackBuiltinPlugin;
	options: Options;
	resolvedOptions: ResolvedOptions;
	constructor(options: Options = {}) {
		this.plugin = new RawRSCClientReferenceManifestRspackPlugin();
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
			test: [/\.(j|t|mj|cj)sx?$/i],
			exclude: this.options.exclude ?? {
				// Exclude libraries in node_modules ...
				and: [/node_modules/]
			},
			use: [
				{
					loader: "builtin:rsc-client-entry-loader",
					options: this.resolvedOptions
				}
			]
		});
	}
	resolveOptions(compiler: Compiler): ResolvedOptions {
		const entry = Object.assign({}, compiler.options.entry);
		const resolvedEntry: Record<string, string> = {};
		const root = compiler.options.context ?? process.cwd();
		for (let item of Object.keys(entry)) {
			const imports = entry[item].import;
			if (imports) {
				resolvedEntry[item] = imports[0];
			}
		}
		const resolvedRoutes = this.options.routes ?? [];
		const output = path.resolve(root, "./dist/server");
		return {
			entry: resolvedEntry,
			root: output,
			routes: resolvedRoutes
		};
	}
}

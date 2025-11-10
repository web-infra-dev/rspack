import type {
	BuiltinPlugin,
	RawOptimizeDependencyReferencedExportsPluginOptions
} from "@rspack/binding";
import { BuiltinPluginName } from "@rspack/binding";

import {
	createBuiltinPlugin,
	RspackBuiltinPlugin
} from "../builtin-plugin/base";
import {
	getFileName,
	type ModuleFederationManifestPluginOptions
} from "../container/ModuleFederationManifestPlugin";
import type { SharedConfig } from "./SharePlugin";

type OptimizeSharedConfig = {
	shareKey: string;
	treeshake: boolean;
	usedExports?: string[];
};

export class OptimizeDependencyReferencedExportsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.OptimizeDependencyReferencedExportsPlugin;
	private sharedOptions: [string, SharedConfig][];
	private injectUsedExports: boolean;
	private manifestOptions: ModuleFederationManifestPluginOptions;

	constructor(
		sharedOptions: [string, SharedConfig][],
		injectUsedExports?: boolean,
		manifestOptions?: ModuleFederationManifestPluginOptions
	) {
		super();
		this.sharedOptions = sharedOptions;
		this.injectUsedExports = injectUsedExports ?? true;
		this.manifestOptions = manifestOptions ?? {};
	}

	private buildOptions(): RawOptimizeDependencyReferencedExportsPluginOptions {
		const shared: OptimizeSharedConfig[] = this.sharedOptions.map(
			([shareKey, config]) => ({
				shareKey,
				treeshake: !!config.treeshake,
				usedExports: config.usedExports
			})
		);
		const { manifestFileName, statsFileName } = getFileName(
			this.manifestOptions
		);
		return {
			shared,
			injectUsedExports: this.injectUsedExports,
			manifestFileName,
			statsFileName
		};
	}

	raw(): BuiltinPlugin | undefined {
		if (!this.sharedOptions.length) {
			return;
		}
		return createBuiltinPlugin(this.name, this.buildOptions());
	}
}

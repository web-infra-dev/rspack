import type { Compiler } from "../Compiler";
import type { Plugins } from "../config";
import type { ModuleFederationPluginOptions } from "../container/ModuleFederationPlugin";
import { IndependentSharedPlugin } from "./IndependentSharedPlugin";
import { SharedUsedExportsOptimizerPlugin } from "./SharedUsedExportsOptimizerPlugin";
import { normalizeSharedOptions } from "./SharePlugin";

export interface TreeshakeSharedPluginOptions {
	mfConfig: ModuleFederationPluginOptions;
	plugins?: Plugins;
	reShake?: boolean;
}

export class TreeShakeSharedPlugin {
	mfConfig: ModuleFederationPluginOptions;
	outputDir: string;
	plugins?: Plugins;
	reShake?: boolean;
	private _independentSharePlugin?: IndependentSharedPlugin;

	name = "TreeShakeSharedPlugin";
	constructor(options: TreeshakeSharedPluginOptions) {
		const { mfConfig, plugins, reShake } = options;
		this.mfConfig = mfConfig;
		this.outputDir = mfConfig.independentShareDir || "independent-packages";
		this.plugins = plugins;
		this.reShake = Boolean(reShake);
	}

	apply(compiler: Compiler) {
		const { mfConfig, outputDir, plugins, reShake } = this;
		const { name, shared, library } = mfConfig;
		if (!shared) {
			return;
		}
		const sharedOptions = normalizeSharedOptions(shared);
		if (!sharedOptions.length) {
			return;
		}

		if (
			sharedOptions.some(
				([_, config]) => config.treeshake && config.import !== false
			)
		) {
			if (!reShake) {
				new SharedUsedExportsOptimizerPlugin(
					sharedOptions,
					mfConfig.injectUsedExports,
					mfConfig.manifest
				).apply(compiler);
			}
			this._independentSharePlugin = new IndependentSharedPlugin({
				name: name,
				shared: shared,
				outputDir,
				plugins,
				treeshake: reShake,
				library,
				manifest: mfConfig.manifest,
				treeshakeSharedExcludePlugins: mfConfig.treeshakeSharedExcludePlugins
			});
			this._independentSharePlugin.apply(compiler);
		}
	}

	get buildAssets() {
		return this._independentSharePlugin?.buildAssets || {};
	}
}

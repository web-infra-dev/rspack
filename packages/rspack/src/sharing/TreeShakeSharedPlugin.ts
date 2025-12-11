import type { Compiler } from "../Compiler";
import type { Plugins } from "../config";
import type { ModuleFederationPluginOptions } from "../container/ModuleFederationPlugin";
import { IndependentSharedPlugin } from "./IndependentSharedPlugin";
import { SharedUsedExportsOptimizerPlugin } from "./SharedUsedExportsOptimizerPlugin";
import { normalizeSharedOptions } from "./SharePlugin";

export interface TreeshakeSharedPluginOptions {
	mfConfig: ModuleFederationPluginOptions;
	plugins?: Plugins;
	reshake?: boolean;
}

export class TreeShakeSharedPlugin {
	mfConfig: ModuleFederationPluginOptions;
	outputDir: string;
	plugins?: Plugins;
	reshake?: boolean;
	private _independentSharePlugin?: IndependentSharedPlugin;

	name = "TreeShakeSharedPlugin";
	constructor(options: TreeshakeSharedPluginOptions) {
		const { mfConfig, plugins, reshake } = options;
		this.mfConfig = mfConfig;
		this.outputDir = mfConfig.independentShareDir || "independent-packages";
		this.plugins = plugins;
		this.reshake = Boolean(reshake);
	}

	apply(compiler: Compiler) {
		const { mfConfig, outputDir, plugins, reshake } = this;
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
			if (!reshake) {
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
				treeshake: reshake,
				library,
				manifest: mfConfig.manifest,
				treeshakeSharedExcludedPlugins: mfConfig.treeshakeSharedExcludedPlugins
			});
			this._independentSharePlugin.apply(compiler);
		}
	}

	get buildAssets() {
		return this._independentSharePlugin?.buildAssets || {};
	}
}

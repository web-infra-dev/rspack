import type { BuiltinPlugin } from "@rspack/binding";
import { BuiltinPluginName } from "@rspack/binding";

import {
	createBuiltinPlugin,
	RspackBuiltinPlugin
} from "../builtin-plugin/base";
import type { SharedConfig } from "./SharePlugin";

type OptimizeSharedConfig = {
	shareKey: string;
	treeshake: boolean;
	usedExports?: string[];
};

type OptimizeDependencyReferencedExportsOptions = {
	shared: OptimizeSharedConfig[];
	injectUsedExports?: Boolean;
};

export class OptimizeDependencyReferencedExportsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.OptimizeDependencyReferencedExportsPlugin;
	private sharedOptions: [string, SharedConfig][];
	private injectUsedExports: Boolean;

	constructor(
		sharedOptions: [string, SharedConfig][],
		injectUsedExports?: Boolean
	) {
		super();
		this.sharedOptions = sharedOptions;
		this.injectUsedExports = injectUsedExports ?? true;
	}

	private buildOptions(): OptimizeDependencyReferencedExportsOptions {
		const shared: OptimizeSharedConfig[] = this.sharedOptions.map(
			([shareKey, config]) => ({
				shareKey,
				treeshake: !!config.treeshake,
				usedExports: config.usedExports
			})
		);
		return {
			shared,
			injectUsedExports: this.injectUsedExports
		};
	}

	raw(): BuiltinPlugin | undefined {
		if (!this.sharedOptions.length) {
			return;
		}
		return createBuiltinPlugin(this.name, this.buildOptions());
	}
}

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
	ignoredRuntime: string[];
};

export default class OptimizeDependencyReferencedExportsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.OptimizeDependencyReferencedExportsPlugin;
	private sharedOptions: [string, SharedConfig][];
	private ignoredRuntime: string[];

	constructor(
		sharedOptions: [string, SharedConfig][],
		ignoredRuntime?: string[]
	) {
		super();
		this.sharedOptions = sharedOptions;
		this.ignoredRuntime = ignoredRuntime ?? [];
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
			ignoredRuntime: this.ignoredRuntime
		};
	}

	raw(): BuiltinPlugin | undefined {
		if (!this.sharedOptions.length) {
			return;
		}
		return createBuiltinPlugin(this.name, this.buildOptions());
	}
}

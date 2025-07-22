import util from "node:util";
import binding from "@rspack/binding";
import type { Source } from "webpack-sources";

const $assets: unique symbol = Symbol("assets");

declare module "@rspack/binding" {
	interface Assets {
		[$assets]: Record<string, Source>;
	}

	interface KnownBuildInfo {
		assets: Record<string, Source>;
		fileDependencies: Set<string>;
		contextDependencies: Set<string>;
		missingDependencies: Set<string>;
		buildDependencies: Set<string>;
	}
}

Object.defineProperty(binding.KnownBuildInfo.prototype, util.inspect.custom, {
	enumerable: true,
	configurable: true,
	value(this: binding.KnownBuildInfo): any {
		return {
			...this,
			assets: this.assets,
			fileDependencies: this.fileDependencies,
			contextDependencies: this.contextDependencies,
			missingDependencies: this.missingDependencies,
			buildDependencies: this.buildDependencies
		};
	}
});

Object.defineProperty(binding.KnownBuildInfo.prototype, "assets", {
	enumerable: true,
	configurable: true,
	get(this: binding.KnownBuildInfo): Record<string, Source> {
		if (this[binding.BUILD_INFO_ASSETS_SYMBOL][$assets]) {
			return this[binding.BUILD_INFO_ASSETS_SYMBOL][$assets];
		}
		const assets = new Proxy(Object.create(null), {
			ownKeys: () => {
				return this[binding.BUILD_INFO_ASSETS_SYMBOL].keys();
			},
			getOwnPropertyDescriptor() {
				return {
					enumerable: true,
					configurable: true
				};
			}
		}) as Record<string, Source>;
		Object.defineProperty(this[binding.BUILD_INFO_ASSETS_SYMBOL], $assets, {
			enumerable: false,
			configurable: true,
			value: assets
		});
		return assets;
	}
});

Object.defineProperty(binding.KnownBuildInfo.prototype, "fileDependencies", {
	enumerable: true,
	configurable: true,
	get(this: binding.KnownBuildInfo): Set<string> {
		return new Set(this[binding.BUILD_INFO_FILE_DEPENDENCIES_SYMBOL]);
	}
});

Object.defineProperty(binding.KnownBuildInfo.prototype, "contextDependencies", {
	enumerable: true,
	configurable: true,
	get(this: binding.KnownBuildInfo): Set<string> {
		return new Set(this[binding.BUILD_INFO_CONTEXT_DEPENDENCIES_SYMBOL]);
	}
});

Object.defineProperty(binding.KnownBuildInfo.prototype, "missingDependencies", {
	enumerable: true,
	configurable: true,
	get(this: binding.KnownBuildInfo): Set<string> {
		return new Set(this[binding.BUILD_INFO_MISSING_DEPENDENCIES_SYMBOL]);
	}
});

Object.defineProperty(binding.KnownBuildInfo.prototype, "buildDependencies", {
	enumerable: true,
	configurable: true,
	get(this: binding.KnownBuildInfo): Set<string> {
		return new Set(this[binding.BUILD_INFO_BUILD_DEPENDENCIES_SYMBOL]);
	}
});

export type { BuildInfo } from "@rspack/binding";

const knownBuildInfoFields: Set<string> = new Set([
	"assets",
	"fileDependencies",
	"contextDependencies",
	"missingDependencies",
	"buildDependencies"
]);

export const commitCustomFieldsToRust = (buildInfo: binding.BuildInfo) => {
	// Sync custom buildInfo fields to the Rust side for later persistent caching by Rust.
	if (Object.keys(buildInfo).some(key => !knownBuildInfoFields.has(key))) {
		buildInfo[binding.COMMIT_CUSTOM_FIELDS_SYMBOL]();
	}
};

import * as binding from "@rspack/binding";
import type { Source } from "webpack-sources";
import { DependenciesBlock } from "./DependenciesBlock";
import { JsSource } from "./util/source";

Object.defineProperty(binding.ConcatenatedModule.prototype, "blocks", {
	enumerable: true,
	configurable: true,
	get(this: binding.ConcatenatedModule) {
		return this._blocks.map(block => DependenciesBlock.__from_binding(block));
	}
});
Object.defineProperty(binding.ConcatenatedModule.prototype, "originalSource", {
	enumerable: true,
	configurable: true,
	value(this: binding.ConcatenatedModule) {
		const originalSource = this._originalSource();
		if (originalSource) {
			return JsSource.__from_binding(originalSource);
		}
		return null;
	}
});
Object.defineProperty(binding.ConcatenatedModule.prototype, "emitFile", {
	enumerable: true,
	configurable: true,
	value(
		this: binding.ConcatenatedModule,
		filename: string,
		source: Source,
		assetInfo?: binding.AssetInfo
	) {
		return this._emitFile(filename, JsSource.__to_binding(source), assetInfo);
	}
});

declare module "@rspack/binding" {
	interface ConcatenatedModule {
		get blocks(): DependenciesBlock[];
		originalSource(): Source | null;
		emitFile(filename: string, source: Source, assetInfo?: AssetInfo): void;
	}
}

export { ConcatenatedModule } from "@rspack/binding";

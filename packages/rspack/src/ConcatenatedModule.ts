import * as binding from "@rspack/binding";
import type { Source } from "webpack-sources";
import { DependenciesBlock } from "./DependenciesBlock";
import { JsSource } from "./util/source";

if (!binding.ConcatenatedModule.prototype.hasOwnProperty("blocks")) {
	Object.defineProperty(binding.ConcatenatedModule.prototype, "blocks", {
		enumerable: true,
		get(this: binding.ConcatenatedModule) {
			return this._blocks.map(block => DependenciesBlock.__from_binding(block));
		}
	});
}
if (!binding.ConcatenatedModule.prototype.hasOwnProperty("originalSource")) {
	Object.defineProperty(
		binding.ConcatenatedModule.prototype,
		"originalSource",
		{
			enumerable: true,
			value(this: binding.ConcatenatedModule) {
				return null;
			}
		}
	);
}
if (!binding.ConcatenatedModule.prototype.hasOwnProperty("emitFile")) {
	Object.defineProperty(binding.ConcatenatedModule.prototype, "emitFile", {
		enumerable: true,
		value(
			this: binding.ConcatenatedModule,
			filename: string,
			source: Source,
			assetInfo?: binding.AssetInfo
		) {
			return this._emitFile(filename, JsSource.__to_binding(source), assetInfo);
		}
	});
}

declare interface ConcatenatedModule extends binding.ConcatenatedModule {
	buildInfo: Record<string, any>;
	buildMeta: Record<string, any>;
	get blocks(): DependenciesBlock[];
	originalSource(): Source | null;
	emitFile(
		filename: string,
		source: Source,
		assetInfo?: binding.AssetInfo
	): void;
}

export const ConcatenatedModule =
	binding.NormalModule as unknown as ConcatenatedModule;

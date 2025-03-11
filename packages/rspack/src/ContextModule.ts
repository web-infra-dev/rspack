import { type AssetInfo, ContextModule } from "@rspack/binding";
import type { Source } from "webpack-sources";
import { DependenciesBlock } from "./DependenciesBlock";
import { JsSource } from "./util/source";

if (!ContextModule.prototype.hasOwnProperty("blocks")) {
	Object.defineProperty(ContextModule.prototype, "blocks", {
		enumerable: true,
		get(this: ContextModule) {
			return this._blocks.map(block => DependenciesBlock.__from_binding(block));
		}
	});
}
if (!ContextModule.prototype.hasOwnProperty("originalSource")) {
	Object.defineProperty(ContextModule.prototype, "originalSource", {
		enumerable: true,
		value(this: ContextModule) {
			return null;
		}
	});
}
if (!ContextModule.prototype.hasOwnProperty("emitFile")) {
	Object.defineProperty(ContextModule.prototype, "emitFile", {
		enumerable: true,
		value(
			this: ContextModule,
			filename: string,
			source: Source,
			assetInfo?: AssetInfo
		) {
			return this._emitFile(filename, JsSource.__to_binding(source), assetInfo);
		}
	});
}

declare module "@rspack/binding" {
	interface ContextModule {
		buildInfo: Record<string, any>;
		buildMeta: Record<string, any>;
		get blocks(): DependenciesBlock[];
		originalSource(): Source | null;
		emitFile(filename: string, source: Source, assetInfo?: AssetInfo): void;
	}
}

export { ContextModule } from "@rspack/binding";

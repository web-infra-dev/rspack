import binding from "@rspack/binding";
import type { Source } from "webpack-sources";
import { SourceAdapter } from "./util/source";

Object.defineProperty(binding.ContextModule.prototype, "identifier", {
	enumerable: true,
	configurable: true,
	value(this: binding.Module): string {
		return this[binding.MODULE_IDENTIFIER_SYMBOL];
	}
});
Object.defineProperty(binding.ContextModule.prototype, "originalSource", {
	enumerable: true,
	configurable: true,
	value(this: binding.ContextModule) {
		const originalSource = this._originalSource();
		if (originalSource) {
			return SourceAdapter.fromBinding(originalSource);
		}
		return null;
	}
});
Object.defineProperty(binding.ContextModule.prototype, "emitFile", {
	enumerable: true,
	configurable: true,
	value(
		this: binding.ContextModule,
		filename: string,
		source: Source,
		assetInfo?: binding.AssetInfo
	) {
		return this._emitFile(filename, SourceAdapter.toBinding(source), assetInfo);
	}
});

export { ContextModule } from "@rspack/binding";

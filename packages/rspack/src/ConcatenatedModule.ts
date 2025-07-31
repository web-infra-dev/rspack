import binding from "@rspack/binding";
import type { Source } from "webpack-sources";
import { JsSource } from "./util/source";

Object.defineProperty(binding.ConcatenatedModule.prototype, "identifier", {
	enumerable: true,
	configurable: true,
	value(this: binding.ConcatenatedModule): string {
		return this[binding.MODULE_IDENTIFIER_SYMBOL];
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
		this._emitFile(filename, JsSource.__to_binding(source), assetInfo);
	}
});

export { ConcatenatedModule } from "@rspack/binding";

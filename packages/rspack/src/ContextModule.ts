import * as binding from "@rspack/binding";
import type { Source } from "webpack-sources";
import { JsSource } from "./util/source";

Object.defineProperty(binding.ContextModule.prototype, "originalSource", {
	enumerable: true,
	configurable: true,
	value(this: binding.ContextModule) {
		const originalSource = this._originalSource();
		if (originalSource) {
			return JsSource.__from_binding(originalSource);
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
		return this._emitFile(filename, JsSource.__to_binding(source), assetInfo);
	}
});

export { ContextModule } from "@rspack/binding";

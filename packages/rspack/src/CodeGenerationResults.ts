import * as binding from "@rspack/binding";
import { JsSource } from "./util/source";

Object.defineProperty(binding.Sources.prototype, "get", {
	enumerable: true,
	configurable: true,
	value(this: binding.Sources, sourceType: string) {
		const originalSource = this._get(sourceType);
		if (originalSource) {
			return JsSource.__from_binding(originalSource);
		}
		return null;
	}
});

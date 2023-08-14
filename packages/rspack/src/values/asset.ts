import type { JsAssetInfo } from "@rspack/binding";
import { AssetInfo } from "../Compilation";

export function toJsAssetInfo(info?: AssetInfo): JsAssetInfo {
	return {
		immutable: false,
		minimized: false,
		development: false,
		hotModuleReplacement: false,
		related: {},
		chunkHash: [],
		contentHash: [],
		version: "",
		...info
	};
}

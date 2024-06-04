import type { JsAssetInfo as JsAssetInfoBinding } from "@rspack/binding";

export type AssetInfo = Partial<Omit<JsAssetInfo, "extras">> &
	Record<string, any>;

class JsAssetInfo {
	static __from_binding(jsAssetInfo: JsAssetInfoBinding): AssetInfo {
		const {
			immutable,
			minimized,
			development,
			hotModuleReplacement,
			related,
			chunkHash,
			contentHash,
			javascriptModule,
			sourceFilename,
			extras
		} = jsAssetInfo;
		return {
			...extras, // extras should not overwrite any KnownAssetFields
			immutable,
			minimized,
			development,
			hotModuleReplacement,
			related,
			chunkHash,
			contentHash,
			javascriptModule,
			sourceFilename
		};
	}

	static __to_binding(assetInfo: AssetInfo = {}): JsAssetInfoBinding {
		let {
			immutable = false,
			minimized = false,
			development = false,
			hotModuleReplacement = false,
			related = {},
			chunkHash = [],
			contentHash = [],
			javascriptModule,
			sourceFilename,
			...extras
		} = assetInfo;
		extras = extras ?? {};
		return {
			immutable,
			minimized,
			development,
			hotModuleReplacement,
			related,
			chunkHash,
			contentHash,
			extras
		};
	}
}

export { JsAssetInfo };

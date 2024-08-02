import type { JsAssetInfo as JsAssetInfoBinding } from "@rspack/binding";

export type AssetInfo = Partial<Omit<JsAssetInfoBinding, "extras">> &
	Record<string, any>;

const JsAssetInfo = {
	__from_binding(jsAssetInfo: JsAssetInfoBinding): AssetInfo {
		const {
			immutable,
			minimized,
			development,
			hotModuleReplacement,
			related,
			chunkhash,
			fullhash,
			contenthash,
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
			fullhash,
			chunkhash,
			contenthash,
			javascriptModule,
			sourceFilename
		};
	},

	__to_binding(assetInfo: AssetInfo = {}): JsAssetInfoBinding {
		let {
			immutable = false,
			minimized = false,
			development = false,
			hotModuleReplacement = false,
			related = {},
			fullhash = [],
			chunkhash = [],
			contenthash = [],
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
			fullhash,
			chunkhash,
			contenthash,
			extras,
			javascriptModule,
			sourceFilename
		};
	}
};

export { JsAssetInfo };

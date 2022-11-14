import cssReload from "mini-css-extract-plugin/dist/hmr/hotModuleReplacement.js";

var id = "/css-hmr";
// @ts-ignored
__rspack_runtime__.installedModules[id] =
	// @ts-ignored
	__rspack_runtime__.installedModules[id] ||
	function (module) {
		module.exports = cssReload;
	};

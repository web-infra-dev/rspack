const RUNTIME_MODULE_NAME_MAPPING = {
	"webpack/runtime/auto_public_path": "webpack/runtime/publicPath",
	"webpack/runtime/public_path": "webpack/runtime/publicPath",
	"webpack/runtime/async_module": "webpack/runtime/async module",
	"webpack/runtime/base_uri": "webpack/runtime/base uri",
	"webpack/runtime/chunk_name": "webpack/runtime/chunkName",
	"webpack/runtime/compat_get_default_export":
		"webpack/runtime/compat get default export",
	"webpack/runtime/compat": "webpack/runtime/compat",
	"webpack/runtime/create_fake_namespace_object":
		"webpack/runtime/create fake namespace object",
	"webpack/runtime/create_script": "webpack/runtime/trusted types script",
	"webpack/runtime/define_property_getters":
		"webpack/runtime/define property getters",
	"webpack/runtime/ensure_chunk": "webpack/runtime/ensure chunk",
	"webpack/runtime/get_full_hash": "webpack/runtime/getFullHash",
	"webpack/runtime/get_trusted_types_policy":
		"webpack/runtime/trusted types policy",
	"webpack/runtime/global": "webpack/runtime/global",
	"webpack/runtime/has_own_property":
		"webpack/runtime/hasOwnProperty shorthand",
	"webpack/runtime/load_script": "webpack/runtime/load script",
	"webpack/runtime/make_namespace_object":
		"webpack/runtime/make namespace object",
	"webpack/runtime/nonce": "webpack/runtime/nonce",
	"webpack/runtime/on_chunk_loaded": "webpack/runtime/chunk loaded",
	"webpack/runtime/relative_url": "webpack/runtime/relative url",
	"webpack/runtime/runtime_id": "webpack/runtime/runtimeId",
	"webpack/runtime/startup_chunk_dependencies":
		"webpack/runtime/startup chunk dependencies",
	"webpack/runtime/startup_entrypoint": "webpack/runtime/startup entrypoint",
	"webpack/runtime/system_context": "webpack/runtime/__system_context__",
	"webpack/runtime/chunk_prefetch_startup": "webpack/runtime/startup prefetch",
	"webpack/runtime/chunk_prefetch_trigger":
		"webpack/runtime/chunk prefetch trigger",
	"webpack/runtime/chunk_preload_trigger":
		"webpack/runtime/chunk preload trigger",
	"webpack/runtime/css_loading": "webpack/runtime/css loading",
	"webpack/runtime/async_wasm_loading": "webpack/runtime/wasm loading",
	"webpack/runtime/hot_module_replacement":
		"webpack/runtime/hot module replacement",
	"webpack/runtime/readfile_chunk_loading":
		"webpack/runtime/readFile chunk loading",
	"webpack/runtime/require_chunk_loading":
		"webpack/runtime/require chunk loading",
	"webpack/runtime/import_scripts_chunk_loading":
		"webpack/runtime/importScripts chunk loading",
	"webpack/runtime/module_chunk_loading":
		"webpack/runtime/import chunk loading",
	"webpack/runtime/export_webpack_runtime":
		"webpack/runtime/export webpack runtime",
	"webpack/runtime/jsonp_chunk_loading": "webpack/runtime/jsonp chunk loading",
	"webpack/runtime/remote": "webpack/runtime/remotes loading",
	"webpack/runtime/share": "webpack/runtime/sharing",
	"webpack/runtime/consume_shared": "webpack/runtime/consumes",
	"webpack/runtime/harmony_module_decorator":
		"webpack/runtime/harmony module decorator",
	"webpack/runtime/node_module_decorator":
		"webpack/runtime/node module decorator",
	// module name with parameters
	"webpack/runtime/get_chunk_filename": "webpack/runtime/get $1 chunk filename",
	"webpack/runtime/get_main_filename": "webpack/runtime/get $1 filename",
	"webpack/runtime/chunk_prefetch_function": "webpack/runtime/chunk $1 function"
};

const RUNTIME_MODULE_PARAM_REGEX = {
	"webpack/runtime/get_chunk_filename":
		/webpack\/runtime\/get_chunk_filename\/([\w.\-_\s]+)(\*\/)?/g,
	"webpack/runtime/get_main_filename":
		/webpack\/runtime\/get_main_filename\/([\w.\-_\s]+)(\*\/)?/g,
	"webpack/runtime/chunk_prefetch_function":
		/webpack\/runtime\/chunk_prefetch_function\/([\w.\-_\s]+)(\*\/)?/g
};

export function replaceRuntimeModuleName(raw: string) {
	for (const [rspackName, webpackName] of Object.entries(
		RUNTIME_MODULE_NAME_MAPPING
	)) {
		if (
			RUNTIME_MODULE_PARAM_REGEX[
				rspackName as keyof typeof RUNTIME_MODULE_PARAM_REGEX
			]
		) {
			raw = raw.replace(
				RUNTIME_MODULE_PARAM_REGEX[
					rspackName as keyof typeof RUNTIME_MODULE_PARAM_REGEX
				],
				(full, $1, $2) => {
					return webpackName.replace("$1", $1.trim()) + ($2 ? " */" : "");
				}
			);
		} else {
			raw = raw.split(rspackName).join(webpackName);
		}
	}
	return raw;
}

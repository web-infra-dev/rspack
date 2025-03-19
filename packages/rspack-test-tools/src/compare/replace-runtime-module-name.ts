const RUNTIME_MODULE_REGEX = /(webpack\/runtime\/)([a-z_]+)/g;
const RUNTIME_MODULE_NAME_REPLACER: Record<string, string> = {
	auto_public_path: "publicPath",
	public_path: "publicPath",
	async_module: "async module",
	base_uri: "base uri",
	chunk_name: "chunkName",
	compat_get_default_export: "compat get default export",
	compat: "compat",
	create_fake_namespace_object: "create fake namespace object",
	create_script_url: "trusted types script url",
	create_script: "trusted types script",
	define_property_getters: "define property getters",
	ensure_chunk: "ensure chunk",
	get_full_hash: "getFullHash",
	get_trusted_types_policy: "trusted types policy",
	global: "global",
	has_own_property: "hasOwnProperty shorthand",
	load_script: "load script",
	make_namespace_object: "make namespace object",
	nonce: "nonce",
	on_chunk_loaded: "chunk loaded",
	relative_url: "relative url",
	runtime_id: "runtimeId",
	startup_chunk_dependencies: "startup chunk dependencies",
	startup_entrypoint: "startup entrypoint",
	system_context: "__system_context__",
	chunk_prefetch_startup: "startup prefetch",
	chunk_prefetch_trigger: "chunk prefetch trigger",
	chunk_preload_trigger: "chunk preload trigger",
	css_loading: "css loading",
	async_wasm_loading: "wasm loading",
	hot_module_replacement: "hot module replacement",
	readfile_chunk_loading: "readFile chunk loading",
	require_chunk_loading: "require chunk loading",
	import_scripts_chunk_loading: "importScripts chunk loading",
	module_chunk_loading: "import chunk loading",
	export_webpack_runtime: "export webpack runtime",
	jsonp_chunk_loading: "jsonp chunk loading",
	remote: "remotes loading",
	share: "sharing",
	consume_shared: "consumes",
	esm_module_decorator: "harmony module decorator",
	node_module_decorator: "node module decorator"
};

const RUNTIME_MODULE_NAME_MAPPING = {
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

export function replaceRuntimeModuleName(content: string) {
	let res = content.replace(
		RUNTIME_MODULE_REGEX,
		(_, $1: string, $2: string) =>
			`${$1}${RUNTIME_MODULE_NAME_REPLACER[$2] || $2}`
	);
	res = Object.entries(RUNTIME_MODULE_NAME_MAPPING).reduce(
		(res, [rspackName, webpackName]) => {
			if (
				RUNTIME_MODULE_PARAM_REGEX[
					rspackName as keyof typeof RUNTIME_MODULE_PARAM_REGEX
				]
			) {
				return res.replace(
					RUNTIME_MODULE_PARAM_REGEX[
						rspackName as keyof typeof RUNTIME_MODULE_PARAM_REGEX
					],
					(_, $1, $2) => {
						return webpackName.replace("$1", $1.trim()) + ($2 ? " */" : "");
					}
				);
			}

			return res.replaceAll(rspackName, webpackName);
		},
		res
	);
	return res;
}

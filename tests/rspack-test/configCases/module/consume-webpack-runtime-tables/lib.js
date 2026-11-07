var __webpack_modules__ = { prebuilt: () => 42 };
var __rspack_modules = { prebuilt: () => 7 };
const __webpack_module_cache__ = { value: __webpack_modules__.prebuilt() };
const __rspack_module_cache = { value: __rspack_modules.prebuilt() };

export const moduleIds = [
	Object.keys(__webpack_modules__),
	Object.keys(__rspack_modules)
];
export const values = [
	__webpack_module_cache__.value,
	__rspack_module_cache.value
];

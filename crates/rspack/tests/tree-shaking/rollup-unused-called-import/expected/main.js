(function() {// mount Modules
(function () {
	runtime.installedModules = {
"./dead.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
},
"./foo.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
function _default() {
    return "foo";
}
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _foo = __rspack_runtime__.interopRequire(__rspack_require__("./foo.js"));
assert.equal((0, _foo.default)(), "foo");
},

};
})();

// mount Chunks
(function () {
	runtime.installedChunks = {};
})();

// mount ModuleCache
(function () {
	runtime.moduleCache = {};
})();
self["__rspack_runtime__"].__rspack_require__("./index.js");})()
(function() {// mount Modules
(function () {
	runtime.installedModules = {
"./bar.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "b", {
    enumerable: true,
    get: ()=>b
});
__rspack_runtime__.exportStar(__rspack_require__("./foo.js"), exports);
__rspack_runtime__.exportStar(__rspack_require__("./result.js"), exports);
function b() {}
},
"./foo.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: ()=>a
});
__rspack_runtime__.exportStar(__rspack_require__("./bar.js"), exports);
__rspack_runtime__.exportStar(__rspack_require__("./result.js"), exports);
const a = 3;
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__rspack_runtime__.exportStar(__rspack_require__("./foo.js"), exports);
__rspack_runtime__.exportStar(__rspack_require__("./bar.js"), exports);
__rspack_runtime__.exportStar(__rspack_require__("./result.js"), exports);
},
"./result.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "c", {
    enumerable: true,
    get: ()=>c
});
__rspack_runtime__.exportStar(__rspack_require__("./foo.js"), exports);
__rspack_runtime__.exportStar(__rspack_require__("./bar.js"), exports);
const c = 103330;
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
self["__rspack_runtime__"].__rspack_register__(["main"], {
"./app.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "Provider", {
    enumerable: true,
    get: ()=>_lib.default
});
const _lib = __rspack_runtime__.interopRequire(__rspack_require__("./lib.js"));
},
"./foo.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__rspack_runtime__.exportStar(__rspack_require__("./app.js"), exports);
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _foo = __rspack_require__("./foo.js");
_foo.Provider;
},
"./lib.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
function Provider() {}
const _default = Provider;
},
});self["__rspack_runtime__"].__rspack_require__("./index.js");
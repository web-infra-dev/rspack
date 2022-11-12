self["__rspack_runtime__"].__rspack_register__(["main"], {
"./bar.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>test
});
function test() {}
},
"./foo.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "Select", {
    enumerable: true,
    get: ()=>_bar.default
});
const _bar = __rspack_runtime__.interopRequire(__rspack_require__("./bar.js"));
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _foo = __rspack_require__("./foo.js");
(0, _foo.Select)();
},
});self["__rspack_runtime__"].__rspack_require__("./index.js");
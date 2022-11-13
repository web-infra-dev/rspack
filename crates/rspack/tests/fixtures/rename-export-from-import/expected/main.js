self["__rspack_runtime__"].__rspack_register__(["main"], {
"./app.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "q", {
    enumerable: true,
    get: ()=>_lib.question
});
const _lib = __rspack_require__("./lib.js");
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _app = __rspack_require__("./app.js");
_app.q;
},
"./lib.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "question", {
    enumerable: true,
    get: ()=>question
});
const question = "2";
},
});self["__rspack_runtime__"].__rspack_require__("./index.js");
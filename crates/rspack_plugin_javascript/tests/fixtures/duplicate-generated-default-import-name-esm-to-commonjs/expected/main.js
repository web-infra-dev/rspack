self["__rspack_runtime__"].__rspack_register__(["main"], {
"./a/cart.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
const _default = 'cart-a';
},
"./b/cart.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
const _default = 'cart-b';
},
"./c/cart.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
const _default = 'cart-c';
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _cart = __rspack_runtime__.interopRequire(__rspack_require__("./a/cart.js"));
const _cart1 = __rspack_runtime__.interopRequire(__rspack_require__("./b/cart.js"));
const _cart2 = __rspack_runtime__.interopRequire(__rspack_require__("./c/cart.js"));
console.log(_cart.default, _cart1.default, _cart2.default);
},
});self["__rspack_runtime__"].__rspack_require__("./index.js");
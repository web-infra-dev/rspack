self["__rspack_runtime__"].__rspack_register__(["main"], {
"./a/cart.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _default = 'cart-a';
},
"./b/cart.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _default = 'cart-b';
},
"./c/cart.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _default = 'cart-c';
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {
"use strict";
function _interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _cart = _interopRequireDefault(__rspack_require__("./a/cart.js"));
var _cart1 = _interopRequireDefault(__rspack_require__("./b/cart.js"));
var _cart2 = _interopRequireDefault(__rspack_require__("./c/cart.js"));
console.log(_cart.default, _cart1.default, _cart2.default);
},
});self["__rspack_runtime__"].__rspack_require__("./index.js");
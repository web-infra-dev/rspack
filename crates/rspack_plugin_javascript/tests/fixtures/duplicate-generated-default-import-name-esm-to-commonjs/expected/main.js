self["__rspack_runtime__"].__rspack_register__([
    "./a/cart.js"
], {
    "./a/cart.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__) {
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
    }
});
self["__rspack_runtime__"].__rspack_register__([
    "./b/cart.js"
], {
    "./b/cart.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__) {
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
    }
});
self["__rspack_runtime__"].__rspack_register__([
    "./c/cart.js"
], {
    "./c/cart.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__) {
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
    }
});
self["__rspack_runtime__"].__rspack_register__([
    "./index.js"
], {
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__) {
        "use strict";
        function _interopRequireDefault(obj) {
            return obj && obj.__esModule ? obj : {
                default: obj
            };
        }
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        var _cart = _interopRequireDefault(require("./a/cart"));
        var _cart1 = _interopRequireDefault(require("./b/cart"));
        var _cart2 = _interopRequireDefault(require("./c/cart"));
        console.log(_cart.default, _cart1.default, _cart2.default);
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");
self.__rspack_runtime__.__rspack_register__([
    "main"
], {
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: !0
        }), Object.defineProperty(exports, "default", {
            enumerable: !0,
            get: ()=>_default
        });
        var Foo = function() {
            console.log("side effect"), this.isFoo = !0;
        };
        let _default = Foo;
        Foo.prototype = {
            answer: function() {
                return 42;
            }
        };
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: !0
        });
        let _foo = __rspack_runtime__.interopRequire(__rspack_require__("./foo.js"));
        new _foo.default();
    }
}), self.__rspack_runtime__.__rspack_require__("./index.js");

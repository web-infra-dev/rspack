self.__rspack_runtime__.__rspack_register__([
    "main"
], {
    "./dead.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {},
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        function _default() {
            return "foo";
        }
        Object.defineProperty(exports, "__esModule", {
            value: !0
        }), Object.defineProperty(exports, "default", {
            enumerable: !0,
            get: ()=>_default
        }), __rspack_runtime__.interopRequire(__rspack_require__("./dead.js"));
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: !0
        });
        let _foo = __rspack_runtime__.interopRequire(__rspack_require__("./foo.js"));
        assert.equal((0, _foo.default)(), "foo");
    }
}), self.__rspack_runtime__.__rspack_require__("./index.js");

self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./dead.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {},
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "default", {
            enumerable: true,
            get: ()=>_default
        });
        __rspack_runtime__.interopRequire(__rspack_require__("./dead.js"));
        function _default() {
            return "foo";
        }
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _foo = __rspack_runtime__.interopRequire(__rspack_require__("./foo.js"));
        assert.equal((0, _foo.default)(), "foo");
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");

self["__rspack_runtime__"].__rspack_register__([
    "bar_js"
], {
    "./bar.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "default", {
            enumerable: true,
            get: ()=>test
        });
        function test() {}
    }
});

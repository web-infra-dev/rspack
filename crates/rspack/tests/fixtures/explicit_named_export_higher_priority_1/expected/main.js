self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./bar.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {},
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "a", {
            enumerable: true,
            get: ()=>a
        });
        __rspack_runtime__.exportStar(__rspack_require__("./bar.js"), exports);
        const a = "foo";
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _foo = __rspack_require__("./foo.js");
        console.log(_foo.a);
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");

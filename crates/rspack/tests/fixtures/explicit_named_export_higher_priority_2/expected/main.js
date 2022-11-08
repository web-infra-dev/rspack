self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./bar.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {},
    "./baz.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "a", {
            enumerable: true,
            get: ()=>a
        });
        const a = "baz";
    },
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "a", {
            enumerable: true,
            get: ()=>_baz.a
        });
        const _baz = __rspack_require__("./baz.js");
        __rspack_runtime__.exportStar(__rspack_require__("./bar.js"), exports);
    },
    "./main.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _foo = __rspack_require__("./foo.js");
        console.log(_foo.a);
    }
});
self["__rspack_runtime__"].__rspack_require__("./main.js");

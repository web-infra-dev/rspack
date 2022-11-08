self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./bar.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {},
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "b", {
            enumerable: true,
            get: ()=>b
        });
        const b = "foo";
    },
    "./main.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        __rspack_runtime__.exportStar(__rspack_require__("./foo.js"), exports);
        __rspack_runtime__.exportStar(__rspack_require__("./bar.js"), exports);
    }
});
self["__rspack_runtime__"].__rspack_require__("./main.js");

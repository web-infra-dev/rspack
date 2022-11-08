self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./bar.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        function _export(target, all) {
            for(var name in all)Object.defineProperty(target, name, {
                enumerable: true,
                get: all[name]
            });
        }
        _export(exports, {
            b: ()=>b,
            bar: ()=>_foo
        });
        const _foo = __rspack_runtime__.interopRequire(__rspack_require__("./foo.js"));
        __rspack_runtime__.exportStar(__rspack_require__("./result.js"), exports);
        function b() {}
    },
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        function _export(target, all) {
            for(var name in all)Object.defineProperty(target, name, {
                enumerable: true,
                get: all[name]
            });
        }
        _export(exports, {
            a: ()=>a,
            foo: ()=>foo
        });
        __rspack_runtime__.exportStar(__rspack_require__("./bar.js"), exports);
        __rspack_runtime__.exportStar(__rspack_require__("./result.js"), exports);
        const a = 3;
        const foo = 3;
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _foo = __rspack_require__("./foo.js");
        _foo.bar.a;
        (0, _foo.c)();
    },
    "./result.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "c", {
            enumerable: true,
            get: ()=>c
        });
        __rspack_runtime__.exportStar(__rspack_require__("./foo.js"), exports);
        __rspack_runtime__.exportStar(__rspack_require__("./bar.js"), exports);
        const c = 103330;
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");

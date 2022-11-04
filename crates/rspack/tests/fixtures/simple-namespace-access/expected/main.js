self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
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
            foo: ()=>foo,
            bar: ()=>bar
        });
        var foo = "lol";
        var bar = "wut";
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _fooJs = __rspack_runtime__.interopRequire(__rspack_require__("./foo.js"));
        _fooJs.bar();
        _fooJs.foo();
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");

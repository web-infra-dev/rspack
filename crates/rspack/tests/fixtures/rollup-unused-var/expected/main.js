self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "foo", {
            enumerable: true,
            get: ()=>foo
        });
        var foo = "lol";
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _fooJs = __rspack_require__("./foo.js");
        console.log(_fooJs.foo);
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");

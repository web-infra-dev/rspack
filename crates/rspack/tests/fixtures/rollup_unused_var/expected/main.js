self.__rspack_runtime__.__rspack_register__([
    "main"
], {
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: !0
        }), Object.defineProperty(exports, "foo", {
            enumerable: !0,
            get: ()=>foo
        });
        var foo = "lol";
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: !0
        });
        let _fooJs = __rspack_require__("./foo.js");
        console.log(_fooJs.foo);
    }
}), self.__rspack_runtime__.__rspack_require__("./index.js");

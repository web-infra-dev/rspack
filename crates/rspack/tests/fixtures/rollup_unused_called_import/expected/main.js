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
        var obj, foo = {
            value: 1
        };
        obj = foo, obj.value += 1;
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: !0
        });
        let _foo = __rspack_require__("./foo.js");
        assert.equal(_foo.foo.value, 2);
    }
}), self.__rspack_runtime__.__rspack_require__("./index.js");

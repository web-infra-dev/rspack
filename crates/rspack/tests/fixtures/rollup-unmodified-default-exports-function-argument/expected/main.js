self.__rspack_runtime__.__rspack_register__([
    "main"
], {
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        function _export(target, all) {
            for(var name in all)Object.defineProperty(target, name, {
                enumerable: !0,
                get: all[name]
            });
        }
        Object.defineProperty(exports, "__esModule", {
            value: !0
        }), _export(exports, {
            default: ()=>_default,
            bar: ()=>bar
        });
        var foo = function() {
            return 42;
        };
        let _default = foo;
        function bar() {
            return contrivedExample(foo);
        }
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: !0
        });
        let _foo = __rspack_runtime__.interopRequire(__rspack_require__("./foo.js"));
        var answer = (0, _foo.default)();
        (0, _foo.bar)(), console.log(answer);
    }
}), self.__rspack_runtime__.__rspack_require__("./index.js");

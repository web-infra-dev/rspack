self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _mathsJs = __rspack_runtime__.interopRequire(__rspack_require__("./maths.js"));
        console.log(_mathsJs.xxx.test);
        console.log(_mathsJs["square"]);
    },
    "./maths.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
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
            square: ()=>square,
            xxx: ()=>_testJs
        });
        const _testJs = __rspack_runtime__.interopRequire(__rspack_require__("./test.js"));
        function square(x) {
            return x * x;
        }
    },
    "./test.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
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
            test: ()=>test,
            ccc: ()=>ccc
        });
        function test() {}
        function ccc() {}
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");

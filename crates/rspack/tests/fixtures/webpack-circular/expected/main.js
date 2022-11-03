self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _module = __rspack_require__("./module.js");
        expect((0, _module.y)("a")).toBe("okBAA");
    },
    "./inner.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
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
            A: ()=>A,
            B: ()=>B
        });
        function A(s) {
            return s + "A";
        }
        function B(s) {
            return s + "B";
        }
    },
    "./module.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
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
            x: ()=>x,
            y: ()=>y
        });
        const _inner = __rspack_require__("./inner.js");
        function x(type) {
            switch(type){
                case "a":
                    return withA("b");
                case "b":
                    return withB("c");
                case "c":
                    return "ok";
            }
        }
        function y(v) {
            return withA(v);
        }
        function withA(v) {
            const value = x(v);
            return (0, _inner.A)(value);
        }
        function withB(v) {
            const value = x(v);
            return (0, _inner.B)(value);
        }
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");

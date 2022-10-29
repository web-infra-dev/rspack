self.__rspack_runtime__.__rspack_register__([
    "main"
], {
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: !0
        });
        let _stuff = __rspack_require__("./stuff.js");
        function getClass() {
            class MyClass {
            }
            return MyClass;
        }
        (0, _stuff.bar)(), (0, _stuff.baz)()(), console.log(getClass().name);
    },
    "./stuff.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        function _export(target, all) {
            for(var name in all)Object.defineProperty(target, name, {
                enumerable: !0,
                get: all[name]
            });
        }
        function bar() {
            console.log("outer bar");
        }
        function Baz() {
            function bar() {
                console.log("inner bar");
            }
            function bog() {
                console.log("inner bog");
            }
            return bar(), bog;
        }
        Object.defineProperty(exports, "__esModule", {
            value: !0
        }), _export(exports, {
            bar: ()=>bar,
            baz: ()=>Baz
        });
    }
}), self.__rspack_runtime__.__rspack_require__("./index.js");

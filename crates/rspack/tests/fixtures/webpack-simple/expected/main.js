self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _module = __rspack_require__("./module.js");
        it("export should be unused when only unused functions use it", ()=>{
            (0, _module.f1)();
            expect(_module.pureUsed).toBe(42);
            expect((0, _module.fWithDefault)()).toBe(42);
            return __rspack_dynamic_require__([
                "chunk_js"
            ]).then(__rspack_require__.bind(__rspack_require__, "./chunk.js"));
        });
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
            EXPORT: ()=>EXPORT,
            EXPORT3: ()=>EXPORT3,
            EXPORT4: ()=>EXPORT4,
            EXPORT5: ()=>EXPORT5,
            EXPORT6: ()=>EXPORT6
        });
        const EXPORT = 42;
        const EXPORT3 = 42;
        const EXPORT4 = 42;
        const EXPORT5 = ()=>42;
        const EXPORT6 = ()=>42;
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
            f1: ()=>f1,
            f2: ()=>f2,
            pureUsed: ()=>pureUsed,
            fWithDefault: ()=>fWithDefault
        });
        const _inner = __rspack_require__("./inner.js");
        function f1() {}
        function f2() {
            return _inner.EXPORT;
        }
        const f7 = ()=>{
            return (0, _inner.EXPORT5)();
        };
        const f8 = ()=>{
            return (0, _inner.EXPORT6)();
        };
        g5();
        f7(f8());
        f2("fwefe"), f2("efwefa");
        f2(f2(), f2());
        f2(class {
            f() {
                return _inner.EXPORT;
            }
        });
        f2(()=>_inner.EXPORT);
        const pureUsed = _inner.EXPORT3;
        function x1() {
            return _inner.EXPORT2;
        }
        const x2 = function() {
            return x1();
        };
        const x3 = ()=>{
            return x2();
        };
        x3();
        function fWithDefault(r = _inner.EXPORT4) {
            return r;
        }
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");

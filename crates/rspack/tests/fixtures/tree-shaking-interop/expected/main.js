self["__rspack_runtime__"].__rspack_register__([
    "main"
], {
    "./a.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "a", {
            enumerable: true,
            get: ()=>a1
        });
        __rspack_dynamic_require__([
            "bar_js"
        ]).then(__rspack_require__.bind(__rspack_require__, "./bar.js")).then((mod)=>{
            console.log(mod);
        });
        const a1 = "a";
        exports.test = 30;
    },
    "./b.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        module.exports = a = "b";
    },
    "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        if ("production" !== process.env.NODE_ENV) {
            const res = __rspack_require__("./a.js");
            module.exports = res;
        } else {
            const c = __rspack_require__("./b.js");
            module.exports = c;
        }
    },
    "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        const _foo = __rspack_runtime__.interopRequire(__rspack_require__("./foo.js"));
        (0, _foo.default)();
    }
});
self["__rspack_runtime__"].__rspack_require__("./index.js");

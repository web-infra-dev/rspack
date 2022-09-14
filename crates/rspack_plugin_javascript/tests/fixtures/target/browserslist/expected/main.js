self["__rspack_runtime__"].__rspack_register__([
    "main"
  ], {"./index.js":function(module, exports, __rspack_require__, __rspack_dynamic_require__) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "main", {
        enumerable: true,
        get: function() {
            return main;
        }
    });
    require("core-js/modules/es.promise.js");
    require("core-js/modules/es.regexp.exec.js");
    require("core-js/modules/es.string.replace.js");
    async function task() {
        await new Promise((res)=>{
            setTimeout(res, 100);
        });
        return 100;
    }
    async function main() {
        await task();
        console.log("hello world!".replaceAll("o", "t"));
    }
},});this["__rspack_runtime__"].__rspack_require__("./index.js");
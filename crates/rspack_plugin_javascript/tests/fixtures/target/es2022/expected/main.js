"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {
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
async function task() {
    await new Promise((res)=>{
        setTimeout(res, 100);
    });
    return 100;
}
async function main() {
    await task();
    console.log("hello world!");
}
},

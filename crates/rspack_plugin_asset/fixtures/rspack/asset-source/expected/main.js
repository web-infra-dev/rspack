self["__rspack_runtime__"].__rspack_register__(["main"], {
"./data.txt": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
module.exports = "- Isn't Rspack a gamechanging bundler?\n  - Hella yeah!";},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _dataTxt = __rspack_runtime__.interopRequire(__rspack_require__("./data.txt"));
console.log(_dataTxt.default);
},
});self["__rspack_runtime__"].__rspack_require__("./index.js");
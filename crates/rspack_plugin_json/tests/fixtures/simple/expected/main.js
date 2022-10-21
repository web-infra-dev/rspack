self["__rspack_runtime__"].__rspack_register__(["main"], {
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _jsonJson = __rspack_runtime__.interopRequire(__rspack_require__("./json.json"));
console.log(_jsonJson.default);
},
"./json.json": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
module.exports = {
  "hello": "world"
}
;},
});self["__rspack_runtime__"].__rspack_require__("./index.js");
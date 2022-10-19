self["__rspack_runtime__"].__rspack_register__(["main"], {
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {
"use strict";
function _interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _jsonJson = _interopRequireDefault(__rspack_require__("./json.json"));
console.log(_jsonJson.default);
},
"./json.json": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {
"use strict";
module.exports = {
  "hello": "world"
}
;},
});self["__rspack_runtime__"].__rspack_require__("./index.js");
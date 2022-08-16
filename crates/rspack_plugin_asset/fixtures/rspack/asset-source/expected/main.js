self["__rspack_runtime__"].__rspack_register__([
    "main"
  ], {"./data.txt":function (module, exports, __rspack_require__, __rspack_dynamic_require__) {
  "use strict";
  module.exports = "- Isn't Rspack a gamechanging bundler?\n  - Hella yeah!";
},"./index.js":function(module, exports, __rspack_require__, __rspack_dynamic_require__) {
    "use strict";
    function _interopRequireDefault(obj) {
        return obj && obj.__esModule ? obj : {
            default: obj
        };
    }
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _dataTxt = _interopRequireDefault(__rspack_require__("./data.txt"));
    console.log(_dataTxt.default);
},});self["__rspack_runtime__"].__rspack_require__("./index.js");
rs.define("./data.txt", function(__rspack_require__, module, exports) {
  "use strict";
  module.exports = "- Isn't Rspack a gamechanging bundler?\n  - Hella yeah!";
});
rs.define("./index.js", function(__rspack_require__, module, exports) {
    "use strict";
    function _interopRequireDefault(obj) {
        return obj && obj.__esModule ? obj : {
            default: obj
        };
    }
    var _dataTxt = _interopRequireDefault(__rspack_require__("./data.txt"));
    console.log(_dataTxt.default);
});
rs.require("./index.js")
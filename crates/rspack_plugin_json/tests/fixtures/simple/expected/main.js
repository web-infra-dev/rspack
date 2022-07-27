rs.define("./json.json", function(__rspack_require__, module, exports) {
    "use strict";
    module.exports = {
  "hello": "world"
}
;
  });
  rs.define("./index.js", function(__rspack_require__, module, exports) {
    "use strict";
    function _interopRequireDefault(obj) {
        return obj && obj.__esModule ? obj : {
            default: obj
        };
    }
    var _jsonJson = _interopRequireDefault(__rspack_require__("./json.json"));
    console.log(_jsonJson.default);
});
rs.require("./index.js")
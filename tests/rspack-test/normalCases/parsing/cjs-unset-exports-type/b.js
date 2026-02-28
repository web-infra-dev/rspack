const reexport = require("./a")
// set to Flagged
Object.defineProperty(exports, '__esModule', { value: true });
// set to Unset, exports is used directly without member access
Object.keys(reexport).forEach(function (k) {
  if (k !== 'default' && !exports.hasOwnProperty(k)) Object.defineProperty(exports, k, {
    enumerable: true,
    get: function () { return reexport[k]; }
  });
});

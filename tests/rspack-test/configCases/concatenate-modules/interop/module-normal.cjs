function exportValue(exports) {
  module.exports = function () { return 42 }

  module.exports.__esModule = true
  module.exports.default = function () {
    "use strict"
    return this === undefined ? 24 : -1
  }
}

exportValue(exports)

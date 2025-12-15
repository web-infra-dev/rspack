function exportValue(exports) {
  module.exports = function () { return 42 }

  module.exports.__esModule = true
  module.exports.default = () => { return 24 }
}

exportValue(exports)
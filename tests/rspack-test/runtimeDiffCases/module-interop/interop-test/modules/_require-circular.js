const other = require("./require-circular.js");
exports.getNamed = () =>
  other.default === "default" ? "named" : other.default;

const other = require("./require-circular-self.js");
exports.default = "default";
exports.named = other.default === "default" ? "named" : "named-incorrect";

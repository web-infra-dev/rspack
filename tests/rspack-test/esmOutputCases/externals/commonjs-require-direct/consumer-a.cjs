const first = require("external");
const second = require("external");
const viaModuleRequire = module.require("external");
const constructed = new require("external");
const { value } = require("external");

exports.first = first;
exports.second = second;
exports.viaModuleRequire = viaModuleRequire;
exports.constructed = constructed;
exports.destructured = value;
exports.method = require("external").getValue();
exports.named = require("external");
this.fromThis = require("external").value;

"use strict";

const supportsWorker = require("@rspack/test-tools/helper/legacy/supportsWorker");

module.exports = () => supportsWorker() && "TODO: support module.parser.javascript.createRequire";

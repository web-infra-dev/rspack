"use strict";

const supportsRequireInModule = require("@rspack/test-tools/helper/legacy/supportsRequireInModule");

module.exports = () => supportsRequireInModule() && "FIXME: __WEBPACK_EXTERNAL_createRequire is not defined";

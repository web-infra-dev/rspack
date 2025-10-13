"use strict";

const supportsWebAssembly = require("@rspack/test-tools/helper/legacy/supportsWebAssembly");

module.exports = () => supportsWebAssembly() && "FIXME: missing assets";

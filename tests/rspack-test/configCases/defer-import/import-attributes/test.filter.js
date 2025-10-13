"use strict";

const supportsTextDecoder = require("@rspack/test-tools/helper/legacy/supportsTextDecoder");

module.exports = () => supportsTextDecoder() && "TODO: support webpack defer import";

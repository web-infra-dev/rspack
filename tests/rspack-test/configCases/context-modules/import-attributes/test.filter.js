"use strict";

const supportsTextDecoder = require("@rspack/test-tools/helper/legacy/supportsTextDecoder");

module.exports = () => supportsTextDecoder() && "TODO: support import with type bytes";

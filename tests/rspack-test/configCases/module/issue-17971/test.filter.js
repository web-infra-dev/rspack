"use strict";

const supportsRequireInModule = require("@rspack/test-tools/helper/legacy/supportsRequireInModule");

module.exports = () => supportsRequireInModule() && "FIXME: Cannot use 'import.meta' outside a module";

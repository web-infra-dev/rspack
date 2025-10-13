"use strict";

const supportsWorker = require("@rspack/test-tools/helper/legacy/supportsWorker");

module.exports = () => supportsWorker() && "FIXME: import() was not found in the worker code for loading async chunks";

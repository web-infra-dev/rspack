const fs = require("fs");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  findBundle: function (i, options) {
    return ["bundle0.js", "bundle1.js"];
  },
};

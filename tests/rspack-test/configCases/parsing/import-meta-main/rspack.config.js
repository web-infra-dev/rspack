"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = [true, false].map((concatenateModules) => {
    return {
        target: "node",
        optimization: {
            concatenateModules,
        }
    };
});

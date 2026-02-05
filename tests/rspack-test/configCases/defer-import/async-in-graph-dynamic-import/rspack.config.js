"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: [`async-node${process.versions.node.split(".").map(Number)[0]}`],
	mode: "none",
	experiments: {
		deferImport: true
	}
};

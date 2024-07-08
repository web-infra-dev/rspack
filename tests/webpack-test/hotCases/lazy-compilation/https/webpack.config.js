"use strict";

const fs = require("fs");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		lazyCompilation: {
			cacheable: false,
			entries: false,
			backend: {
				server: {
					key: fs.readFileSync(path.join(__dirname, "key.pem")),
					cert: fs.readFileSync(path.join(__dirname, "cert.pem"))
				}
			}
		}
	}
};

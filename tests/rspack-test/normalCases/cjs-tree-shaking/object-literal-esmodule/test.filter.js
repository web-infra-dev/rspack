"use strict";

module.exports = function filter(config) {
	return config.mode !== "development";
};

"use strict";

module.exports = warnings =>
	warnings.filter(
		warning =>
			warning.code !== "ModuleParseWarning" ||
			!warning.message.includes("CSS parse warning")
	);

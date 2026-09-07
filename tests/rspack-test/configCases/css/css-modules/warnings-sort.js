"use strict";

module.exports = warnings =>
	warnings.filter(
		warning =>
			warning.code !== "ModuleParseWarning" ||
			warning.message.includes("Broken '@value' at-rule") ||
			warning.message.includes("Missing trailing whitespace")
	);

/** @type {import("@rspack/core").PitchLoaderDefinitionFunction} */
export const pitch = function (remainingRequest) {
	return (
		"module.exports = require(" + JSON.stringify("!!" + remainingRequest) + ");"
	);
};

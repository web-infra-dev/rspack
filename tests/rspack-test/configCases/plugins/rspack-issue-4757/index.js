class Audio {}
exports.Audio = Audio;
const f = function (Audio) {
	Audio.Transcriptions = {};
	Audio.Translations = {};
};
f(Audio);

it("ensure this file minified and run successful", () => {
	const fs = require("fs");
	expect(fs.readFileSync(__filename)).not.toContain("\n");
});

/**
 * The following code is copied from
 * https://github.com/sindresorhus/slash/blob/98b618f5a3bfcb5dd374b204868818845b87bb2f/index.js
 *
 * MIT Licensed
 * Author Sindre Sorhus @sindresorhus
 */
module.exports = function slash(path) {
	const isExtendedLengthPath = path.startsWith("\\\\?\\");

	if (isExtendedLengthPath) {
		return path;
	}
	return path.replace(/\\/g, "/");
};

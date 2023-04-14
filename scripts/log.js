// @ts-nocheck
const PREFIX = "[x] ";

const COLOR = {
	RESET: "\x1b[0m",

	RED: "\x1b[31m",
	GREEN: "\x1b[32m",
	YELLOW: "\x1b[33m"
};

const log = {
	info(message) {
		console.log(PREFIX + COLOR.GREEN + message + COLOR.RESET);
	},
	error(message) {
		console.log(PREFIX + COLOR.RED + message + COLOR.RESET);
	}
};

module.exports = log;

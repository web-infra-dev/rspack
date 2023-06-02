// @ts-nocheck
const PREFIX = "";

const COLOR = {
	RESET: "\x1b[0m",

	RED: "\x1b[31m",
	GREEN: "\x1b[32m",
	YELLOW: "\x1b[33m"
};

export const log = {
	info(message) {
		console.log(PREFIX + COLOR.GREEN + message + COLOR.RESET);
	},
	error(message) {
		console.log(PREFIX + COLOR.RED + message + COLOR.RESET);
	}
};

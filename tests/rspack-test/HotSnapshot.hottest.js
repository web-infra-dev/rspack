const path = require("path");
const { describeByWalk, createHotStepCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp/hot-snapshot`);

/**
 * CSS HMR runtime logs after timers fire ("[HMR] Reload all CSS").
 * Jest treats console output after tests finish as an error, so silence the
 * log unless explicitly requested via PRINT_LOGGER.
 */
const originalConsoleLog = console.log;
beforeAll(() => {
	console.log = (...args) => {
		if (process.env.PRINT_LOGGER === "true") {
			originalConsoleLog.apply(console, args);
		}
	};
});

describeByWalk(__filename, (name, src, dist) => {
	createHotStepCase(name, src, dist, path.join(tempDir, name), "web");
}, {
	source: path.resolve(__dirname, "./hotCases"),
	dist: path.resolve(__dirname, `./js/hot-snapshot`)
});

const fs = require("fs");
globalThis.__logger = {
	debug() {},
	log() {},
	warn() {},
	error() {}
};
__logger.debug("test-console");
__logger.log("test-console");
__logger.warn("test-console");
__logger.error("test-console");

function getTestLogLevels(content) {
	const regex = /__logger\.(\S+?)\("test-console"/g;
	const logs = [];
	while ((array1 = regex.exec(content)) !== null) {
		logs.push(array1[1]);
	}
	return logs;
}

it("should pure funcs", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	const logLevels = getTestLogLevels(content);
	expect(logLevels).toEqual(["debug", "log"]);
});

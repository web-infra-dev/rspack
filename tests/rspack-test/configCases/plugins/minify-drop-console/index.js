const fs = require("fs");

console.debug("test-console");
console.log("test-console");
console.warn("test-console");
console.error("test-console");

function getTestLogLevels(content) {
	const regex = /console\.(\S+?)\("test-console"/g;
	const logs = [];
	while ((array1 = regex.exec(content)) !== null) {
		logs.push(array1[1]);
	}
	return logs;
}

it("should drop all console", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	const logLevels = getTestLogLevels(content);
	expect(logLevels.length).toBe(0);
});

var value = require("../file");
var fs = require("fs");

it("should have runtime chunk", async () => {
	await import('./async.js'); // make sure ensure chunk runtime added
	if (WATCH_STEP === "0") {
		expect(value).toBe(1);
		expect(fs.readdirSync(__dirname).filter(i => i.includes("runtime")).length).toBe(1);
	} else {
		expect(value).toBe(2);
		expect(fs.readdirSync(__dirname).filter(i => i.includes("runtime")).length).toBe(1);
	}
});

import { value, helper } from "./local";
const fs = require("fs");
const path = require("path");
console.log(value);
it("reuses an unchanged unreachable barrel chain", () => {
	const step = Number(WATCH_STEP);
	expect(value).toBe(["cold", "edited", "edited", "final"][step]);
	expect(helper()).toBe("helper");
	const graphLines = JSON.parse(fs.readFileSync(path.join(__dirname, "graph-log.json"), "utf8"));
	if (step === 0) expect(graphLines.some(line => line.startsWith("rebuild chunk graph:"))).toBe(true);
	if (step === 1) {
		const lines = JSON.parse(fs.readFileSync(path.join(__dirname, "cache-log.json"), "utf8"));
		expect(lines.join("\n")).not.toContain("module topology change detected");
		expect(graphLines.some(line => line.startsWith("build chunk graph:"))).toBe(true);
		expect(graphLines.some(line => line.startsWith("rebuild chunk graph:"))).toBe(false);
	}
});

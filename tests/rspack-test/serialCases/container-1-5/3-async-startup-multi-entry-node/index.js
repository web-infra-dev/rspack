const fs = require("fs");
const path = require("path");

const outputFile = path.join(__dirname, "runtime-result.json");

// Clean up from previous runs so the presence of the file is a real signal.
if (fs.existsSync(outputFile)) {
	fs.unlinkSync(outputFile);
}

// Defer the assertion so other entry modules have a chance to run first.
setTimeout(() => {
	const exists = fs.existsSync(outputFile);
	if (!exists) {
		throw new Error(
			"[async-startup-multi-entry] second entry never executed (missing marker file)"
		);
	}
	const data = JSON.parse(fs.readFileSync(outputFile, "utf-8"));
	expect(data).toEqual(["second-entry"]);
}, 0);

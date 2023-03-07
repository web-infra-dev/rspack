const ROOT_DIR = process.env.ROOT_DIR;
const fs = require("fs");
const path = require("path");
class HmrBenchmarkPlugin {
	constructor(options) {
		this.options = options;
		this.durationMap = {};
		// the key of map is the rebuild id, value is the start time stamp
		this.startTimeStampMap = {};
		this.buildId = -1;
	}

	apply(compiler) {
		compiler.hooks.watchRun.tap("HmrBenchmarkPlugin", () => {
			this.buildId++;
			if (this.buildId > 20) {
				// we at most retry 10 times
				process.exit(process.exitCode || -1);
			}
			this.startTimeStampMap[this.buildId] = Date.now();
		});
		compiler.hooks.done.tap("HmrBenchmarkPlugin", stats => {
			if (this.buildId > 0 && this.startTimeStampMap[this.buildId]) {
				let now = Date.now();
				let duration = now - this.startTimeStampMap[this.buildId];
				this.durationMap[this.buildId] = duration;
			}
			if (Object.keys(this.durationMap).length >= 10) {
				let bencherFormat = ConvertBenchmarkDataToBencherFormat(
					this.durationMap
				);
				appendToFile(path.resolve(ROOT_DIR, "output.txt"), bencherFormat);
				process.exit(process.exitCode || 0);
			}
			updateSomething(path.resolve(__dirname, "../src/App.tsx"), this.options);
		});
	}
}

module.exports = HmrBenchmarkPlugin;

function updateSomething(filePath, options) {
	let nextRenderedFile = options[filePath]();
	fs.writeFileSync(filePath, nextRenderedFile);
}

function ConvertBenchmarkDataToBencherFormat(data) {
	let averageMs =
		Object.keys(data).reduce((acc, key) => {
			return data[key] + acc;
		}, 0) / Object.keys(data).length;
	let averageNs = averageMs * 1000000;
	return `hmr_benchmark/react-refresh ... bench:   ${averageNs} ns/iter (+/- 0)`;
}

function appendToFile(outputPath, content) {
	let previousOutputText = fs.readFileSync(outputPath, "utf8").trimEnd();
	let outputText = `${previousOutputText}\n${content}`;
	fs.writeFileSync(outputPath, outputText);
}

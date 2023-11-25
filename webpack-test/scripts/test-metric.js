const process = require("process");
const { isNumber } = require("util");
const { extractTestMetric, renderAllTestsToMarkdown } = require("./test-metric-util");
const isCI = require("is-ci");
const fs = require('fs')
const path = require('path')
let data = "";

process.stdin.on("readable", () => {
	let chunk;
	while (null !== (chunk = process.stdin.read())) {
		data += chunk;
	}
});

process.stdin.on("end", () => {
	// process all the data and write it back to stdout

	// "numFailedTestSuites": 0,
	// "numFailedTests": 0,
  let jsonObj = {}
  try {
    jsonObj = JSON.parse(data)
  } catch(e) {}
	if (isEmptyObject(jsonObj)) {
		process.exit(-1);
	}

	const failedTestSuites = jsonObj.numFailedTestSuites;
	const failedTests = jsonObj.numFailedTests;
	if (!isNumber(failedTestSuites) || !isNumber(failedTests)) {
		// data is broken
		console.error("Failed to get failed data from jest");
		process.exit(-1);
	}
	if (failedTests > 0 || failedTestSuites > 0) {
		process.exit(-1);
	}

	let extractedTestInfo = extractTestMetric(jsonObj);
	let renderedTestMD = renderAllTestsToMarkdown(jsonObj);
	if (!isCI) {
		console.log(renderedTestMD)
		Object.entries(extractedTestInfo).forEach(([k, v]) => {
			console.log(`${k}: ${v}`);
		});
	} else {
    let json = JSON.stringify(extractedTestInfo)
    console.log(json)
    const rootPath = path.resolve(__dirname, "../../")
    fs.writeFileSync(path.resolve(rootPath, "out.json"), json)
		fs.writeFileSync(path.resolve(rootPath, "out.md"), renderedTestMD)
	}
});

const isEmptyObject = (obj) => {
	return (
		obj != undefined && typeof obj === "object" && Object.keys(obj).length === 0
	);
};

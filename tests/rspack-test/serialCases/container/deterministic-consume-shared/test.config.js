const fs = require("fs");
const path = require("path");

const expectedShareKeys = ["alpha", "beta", "delta", "gamma"];

function readAsset(options) {
	return fs.readFileSync(
		path.join(options.output.path, options.output.filename),
		"utf-8"
	);
}

function getConsumeShareKeys(source) {
	const match = source.match(
		/consumesLoadingData\s*=\s*\{\s*chunkMapping:\s*[\s\S]*?,\s*moduleIdToConsumeDataMapping:\s*\{([\s\S]*?)\},\s*initialConsumes:/
	);

	expect(match).toBeTruthy();
	return [...match[1].matchAll(/shareKey: "([^"]+)"/g)].map(
		match => match[1]
	);
}

module.exports = {
	noTests: true,
	afterExecute(options) {
		const [forward, reverse] = options;
		const forwardSource = readAsset(forward);
		const reverseSource = readAsset(reverse);

		expect(getConsumeShareKeys(forwardSource)).toEqual(expectedShareKeys);
		expect(getConsumeShareKeys(reverseSource)).toEqual(expectedShareKeys);
	}
};

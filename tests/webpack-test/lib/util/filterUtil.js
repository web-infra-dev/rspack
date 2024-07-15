const fs = require("fs");

const FilteredStatus = {
	TODO: "TODO",
	PARTIAL_PASS: "PARTIAL_PASS",
	FAILED: "FAILED",
	NO_PLAN: "NO_PLAN",
	PASS: "PASS"
}

function validateFilteredStatus(status) {
	return Object.values(FilteredStatus).includes(status)
}

function createFilteredDescribe(testName, filterPath, config = {}) {
	let flag = true;
	if (fs.existsSync(filterPath)) {
		flag = require(filterPath)(config)
	}
	let shouldRun = flag === true || (Array.isArray(flag) && flag.includes(FilteredStatus.PARTIAL_PASS))
	let filteredName = normalizeFilteredTestName(flag, testName);
	describe.skip(testName, () => {
		it(filteredName, () => { });
	});
	return shouldRun;
}

function normalizeFilterFlag(flag, testName) {
	if (flag === true) {
		return { status: FilteredStatus.PASS, reason: "" };
	}
	if (flag === false) {
		return { status: FilteredStatus.TODO, reason: "TODO" };
	}
	if (flag === -1) {
		return { status: FilteredStatus.NO_PLAN, reason: "No plan" };
	}
	if (typeof flag === 'string') {
		return { status: FilteredStatus.FAILED, reason: flag }
	}
	if (Array.isArray(flag)) {
		const [status, reason = "empty"] = flag;
		if (validateFilteredStatus(status)) {
			return { status, reason }
		}
	}
	throw new Error(`Unvalidate filter flag "${flag}" for "${testName}"`)
}

function encodeFilteredTest(status, reason) {
	return `{{ status = ${status}, reason = ${reason} }}`
}

function decodeFilteredTest(encoded) {
	const regex = /(.*) {{ status = (.*), reason = (.*) }}$/
	const result = encoded.match(regex);
	if (result === null) {
		return result
	}
	const [, fullName, status, reason] = result;
	return { fullName, status, reason }
}

function normalizeFilteredTestName(flag, testName) {
	const { status, reason } = normalizeFilterFlag(flag, testName)
	return encodeFilteredTest(status, reason)
}

module.exports = {
	FilteredStatus,
	decodeFilteredTest,
	normalizeFilteredTestName,
	createFilteredDescribe
};

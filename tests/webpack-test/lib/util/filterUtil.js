const fs = require("fs");

const FilteredStatus = {
	UNCHECKED: "UNCHECKED", // failed with reason that we don't know
	TODO: "TODO", // failed by some features that we don't support
	FIXME: "FIXME", // failed by some bug
	NO_PLAN: "NO_PLAN", // failed by some features that we won't support
	PARTIAL_PASS: "PARTIAL_PASS",
	PASS: "PASS"
}

function validateFilteredStatus(status) {
	return Object.values(FilteredStatus).includes(status)
}

function createFilteredDescribe(testName, filterPath, config = {}) {
	let flag = true;
	if (fs.existsSync(filterPath)) {
		try {
			flag = require(filterPath)(config)
		} catch (e) {
			console.error(`get filter flag failed from '${filterPath}': ${e.message}`)
		}
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
		return { status: FilteredStatus.UNCHECKED, reason: "UNKNOWN" };
	}
	if (typeof flag === 'string') {
		let status = flag.split(":")[0] || "FAILED";
		let reason = flag.split(":")[1] || "UNKNOWN";
		if (status === "TODO") {
			return { status: FilteredStatus.TODO, reason }
		} else if (status === "NOPLAN") {
			return { status: FilteredStatus.NO_PLAN, reason }
		} else if (status === "FIXME") {
			return { status: FilteredStatus.FIXME, reason }
		} else {
			return { status: FilteredStatus.UNCHECKED, reason }
		}
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

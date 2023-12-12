const nullValue = null;
const undefinedValue = undefined;
const falseValue = false;
const zeroValue = 0;
const emptyStringValue = "";

class FailPlugin {
	apply() {
		throw new Error("FailedPlugin");
	}
}

/** @type {import("../../../../src/index").RspackOptions} */
module.exports = {
	plugins: [
		undefinedValue && new FailPlugin(),
		nullValue && new FailPlugin(),
		falseValue && new FailPlugin(),
		zeroValue && new FailPlugin(),
		emptyStringValue && new FailPlugin()
	]
};

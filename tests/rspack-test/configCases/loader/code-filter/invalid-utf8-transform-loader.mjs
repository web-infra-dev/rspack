export const raw = true;

export default function (source) {
	if (!this.data.pitched) {
		throw new Error("The filtered loader pitch function did not run");
	}
	if (!Buffer.isBuffer(source) || source.length !== 1 || source[0] !== 0xff) {
		throw new Error("Expected the original invalid UTF-8 buffer");
	}
	return Buffer.from([0xff, 0x01]);
}

export const pitch = function (_remainingRequest, _previousRequest, data) {
	data.pitched = true;
};

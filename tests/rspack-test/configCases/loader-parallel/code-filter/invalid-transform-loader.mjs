export const raw = true;

export default function (source) {
	if (!this.data.pitched || !Buffer.isBuffer(source) || source.length !== 1 || source[0] !== 0xff) {
		throw new Error("The invalid UTF-8 buffer did not reach the parallel loader");
	}
	return Buffer.from([0xff, 0x01]);
}

export const pitch = function (_remainingRequest, _previousRequest, data) {
	data.pitched = true;
};

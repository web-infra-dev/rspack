export const raw = true;

export default function (source) {
	if (!Buffer.isBuffer(source) || source.length !== 2 || source[0] !== 0xff || source[1] !== 0x01) {
		throw new Error("The parallel loader did not run on invalid UTF-8 input");
	}
	return 'export default "invalid-preserved";';
}

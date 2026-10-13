export const raw = true;

export default function (source) {
	if (!Buffer.isBuffer(source) || source.length !== 1 || source[0] !== 0xff) {
		throw new Error("The invalid UTF-8 input was changed");
	}
	return 'module.exports = "invalid-preserved";';
}

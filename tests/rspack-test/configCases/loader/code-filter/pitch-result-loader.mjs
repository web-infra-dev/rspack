export default function () {
	throw new Error("A pitch result should skip the normal loader function");
}

export const pitch = function () {
	return 'module.exports = "pitch-short-circuit";';
};

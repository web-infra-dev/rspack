const cjs = require("./lib/cjs");
const esm = require("./lib/esm");

exports.getFilePath = async function () {
	const cjsLib = await cjs.getFilePath();
	const esmLib = await esm.getFilePath();
	return ["cjs.js", cjsLib, esmLib].join("|");
};

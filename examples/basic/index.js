const cjs = require("./cjs");
const esm = require("./esm");


(async () => {
	console.log(await cjs.getFilePath())
	console.assert(await cjs.getFilePath() === "cjs.js|lib/cjs.js|lib/esm.js")
	console.assert(await esm.getFilePath() === "esm.js|lib/cjs.js|lib/esm.js")
})()


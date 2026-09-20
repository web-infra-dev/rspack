const { parentPort } = require("worker_threads");
const dependency = require("./dependency.cjs");

this.value = 42;
parentPort.postMessage({
	thisIsExports: this === module.exports,
	value: module.exports.value,
	globalUntouched: !Object.prototype.hasOwnProperty.call(globalThis, "value"),
	dependency
});

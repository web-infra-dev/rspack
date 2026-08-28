const { isMainThread } = require("node:worker_threads");

module.exports = function (source) {
	if (isMainThread) throw new Error("parallel loader ran on the main thread");
	return source.replace('"ok"', '"worker:ok"');
};

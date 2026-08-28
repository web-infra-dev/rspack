const { isMainThread, threadId } = require("node:worker_threads");

module.exports = function () {
	const callback = this.async();
	setTimeout(() => {
		callback(
			isMainThread ? new Error("parallel loader ran on the main thread") : null,
			`module.exports = ${threadId}`
		);
	}, 30);
};

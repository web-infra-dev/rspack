// This test case uses multi compiler, our implementation has not guaranteed for
// execution order, which means stats.error is out of order, so we use a sort
// function to make the stats.error result consistent
module.exports = function (arr) {
	return arr.sort((a, b) => {
		if (a.compilerPath !== b.compilerPath) {
			return a.compilerPath.localeCompare(b.compilerPath);
		}
		// make sure this error message is always at the end of the group `ddd`
		if (a.message.includes('DDD environment variable is undefined.')) {
			return 1
		} else if (b.message.includes('DDD environment variable is undefined.')){
			return -1
		}


		// This sorts the error message by the message without detailed diagnostics.
		// as `error.message` may contain different span locations,
		// thus resulting the final message not stable.
		let messageA = a.message.split(/\r?\n/)[0];
		let messageB = b.message.split(/\r?\n/)[0];
		return messageA.localeCompare(messageB)
	});
};

module.exports = function (arr) {
	return arr.sort((a, b) => {
		if (a.compilerPath !== b.compilerPath) {
			return a.compilerPath.localeCompare(b.compilerPath);
		}
		return a.message.localeCompare(b.message);
	});
};

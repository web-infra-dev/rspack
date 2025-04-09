module.exports = function (content) {
	return new Promise(resolve => {
		setTimeout(() => {
			resolve(content);
		});
	});
};

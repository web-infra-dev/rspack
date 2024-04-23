module.exports = function (content) {
	const callback = this.async();
	setTimeout(() => {
		callback(new Error("Failed to load (async)"));
	}, 100);
};

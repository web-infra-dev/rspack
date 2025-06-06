module.exports = async function () {
	const callback = this.async();
	this.loadModule(this.resourcePath, function (err, source) {
		callback(err, source);
	});
};

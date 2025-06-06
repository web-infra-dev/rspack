module.exports = async function () {
	return await new Promise((res, rej) => {
		this.loadModule(this.resourcePath, function (err, source) {
			if (err) {
				rej(err);
			} else {
				res(source);
			}
		});
	});
};

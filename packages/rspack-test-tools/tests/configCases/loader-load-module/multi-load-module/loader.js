module.exports = async function (code) {
	const loadModule = path => {
		return new Promise((res, rej) => {
			this.loadModule(path, function (err, source) {
				if (err) {
					rej(err);
				} else {
					res(source);
				}
			});
		});
	};
	const [source_1, source_2] = await Promise.all([
		loadModule(this.resourcePath),
		loadModule(this.resourcePath)
	]);

	expect(code).toEqual(source_1);
	expect(code).toEqual(source_2);
	return code;
};

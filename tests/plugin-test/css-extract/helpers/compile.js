module.exports = compiler =>
	new Promise((resolve, reject) => {
		compiler.run((error, stats) => {
			if (error) {
				return reject(error);
			}

			return resolve(stats);
		});
	});

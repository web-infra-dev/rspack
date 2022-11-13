module.exports = function (done, options, callback) {
	return function (err, stats) {
		if (err) {
			return done(err);
		}

		module.hot
			.check(options || true)
			.then(updatedModules => {
				if (!updatedModules) {
					return done(Error("no update available"));
				}
				if (callback) {
					return callback(stats);
				}
			})
			.catch(err => {
				done(err);
			});
	};
};

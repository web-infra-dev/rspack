module.exports = function (done, options, callback) {
	return function (err, stats) {
		if (err) return done(err);
		module.hot
			.check(options || true)
			.then(updatedModules => {
				console.log(__webpack_hash__);
				if (!updatedModules) return done(new Error("No update available"));
				if (callback) callback(stats);
			})
			.catch(err => {
				done(err);
			});
	};
};

__webpack_require__.hmrM = function () {
	return Promise.resolve()
		.then(function () {
			return require("./" + __webpack_require__.hmrF());
		})['catch'](function (err) {
			if (err.code !== 'MODULE_NOT_FOUND') throw err;
		});
};

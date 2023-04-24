__webpack_require__.es = function (from, to) {
	Object.keys(from).forEach(function (k) {
		if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k))
			Object.defineProperty(to, k, {
				enumerable: true,
				get: function () {
					return from[k];
				}
			});
	});
	return from;
};

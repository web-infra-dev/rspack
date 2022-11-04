(function () {
	runtime.checkById = function (obj, prop) {
		return Object.prototype.hasOwnProperty.call(obj, prop);
	};
})();

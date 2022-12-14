for (
	var i = 0,
		getI = (function () {
			return i;
		})();
	i < 3;
	i++
)
	console.log(getI());

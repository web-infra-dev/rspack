(async function () {
	return import("./module").then(function (mod) {
		if (mod.result !== 42) throw new Error("panic");
	});
})();

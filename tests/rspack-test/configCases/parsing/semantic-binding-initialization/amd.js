define(["require", "exports"], function (load, result) {
	var load;
	result.value = load("./value");
	result.local = function (require) {
		return require("./missing-amd-local");
	};
});

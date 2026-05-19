exports.REF = undefined;

exports.getPath = function () {
	return exports.REF?.main?.filename || null;
};

exports.getSimple = function () {
	return exports.REF?.value || "default";
};

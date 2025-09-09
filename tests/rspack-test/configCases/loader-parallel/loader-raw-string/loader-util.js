exports.ensureObject = o => {
	if (o === undefined) {
		return {};
	}

	if (o && typeof o === "object") {
		return o;
	}

	return JSON.parse(o);
};

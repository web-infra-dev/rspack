const d = (object, property, defaultValue) => {
	if (
		typeof object[property] === "undefined" &&
		typeof defaultValue !== "undefined"
	) {
		object[property] = defaultValue;
	}
	return object[property];
};

const normalizeOptions = function (options) {
	d(options, "exclude", /node_modules/i);
	d(options, "include", /\.([cm]js|[jt]sx?|flow)$/i);
	return options;
};

module.exports = { normalizeOptions };

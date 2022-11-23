module.exports = function (context) {
	return {
		content: context.source.getCode().replace("42", "43"),
		map: undefined,
		meta: ""
	};
};

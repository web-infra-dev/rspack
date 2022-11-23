module.exports = function (context) {
	console.log("1", context.source.getCode());
	return {
		content: context.source.getCode().replace("42", "43"),
		map: undefined,
		meta: ""
	};
};

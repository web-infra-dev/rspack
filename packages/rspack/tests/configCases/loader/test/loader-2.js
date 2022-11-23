module.exports = function (context) {
	return {
		content: context.source.getCode().replace("43", "44"),
		map: undefined,
		meta: ""
	};
};

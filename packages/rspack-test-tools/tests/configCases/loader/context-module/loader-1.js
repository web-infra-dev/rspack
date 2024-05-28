module.exports = function (content) {
  const { request, userRequest, rawRequest, buildInfo, buildMeta} = this._module;
	buildInfo.LOADER_ACCESS = true;
	buildMeta.LOADER_ACCESS = true;
	return (
		"module.exports = " +
		JSON.stringify({
			request,
			userRequest,
			rawRequest,
			prev: content,
		})
	);
};

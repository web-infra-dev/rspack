/** @type {import("@rspack/core").LoaderDefinition<{ get(): string }>} */
export const pitch = function (request) {
	return `
    var content = require(${stringifyRequest(this, `!!${request}`)});
    module.exports = content;
    `
};

function stringifyRequest(loaderContext, request) {
    return JSON.stringify(
      loaderContext.utils.contextify(
        loaderContext.context || loaderContext.rootContext,
        request
      )
    )
}

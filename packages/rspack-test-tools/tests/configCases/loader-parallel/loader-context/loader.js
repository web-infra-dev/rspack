module.exports = function () {
	let module = loaderContext._module


	expect(loaderContext).toMatchSnapshot()

	return ""
}

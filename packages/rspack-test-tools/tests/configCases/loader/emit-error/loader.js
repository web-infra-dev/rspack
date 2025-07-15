module.exports = function (content) {
	this.emitError(new Error("error1"))
	const error = new Error("error2")
	error.file = "./index.js:3:1"
	error.module = this._module
	this._compilation.errors.push(error)
	return content
}

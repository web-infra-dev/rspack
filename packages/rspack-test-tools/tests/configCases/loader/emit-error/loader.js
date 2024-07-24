module.exports = function(content) {
	this.emitError(new Error("error1"))
	let error = new Error("error2")
	error.file = "./index.js:3:1"
	error.moduleIdentifier = this._module.identifier()
	this._compilation.errors.push(error)
	return content
}

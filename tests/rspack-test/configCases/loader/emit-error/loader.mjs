export default function (content) {
	this.emitError(new Error("error1"))
	let error = new Error("error2")
	error.file = "./index.js:3:1"
	error.module = this._module
	this._compilation.errors.push(error)
	// This compilation-local side effect requires the loader to run on every build.
	this.cacheable(false)
	return content
}

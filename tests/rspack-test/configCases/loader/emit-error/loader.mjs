export default function (content) {
	// The direct compilation.errors mutation must run on every build.
	this.cacheable(false)
	this.emitError(new Error("error1"))
	let error = new Error("error2")
	error.file = "./index.js:3:1"
	error.module = this._module
	this._compilation.errors.push(error)
	return content
}

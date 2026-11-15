export default function (content) {
	this.emitError(new Error("error1"))
	return content
}

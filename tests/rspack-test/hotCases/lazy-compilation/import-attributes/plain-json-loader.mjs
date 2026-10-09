export default function () {
	throw new Error(`json without import attributes: ${this.resourcePath}`);
}

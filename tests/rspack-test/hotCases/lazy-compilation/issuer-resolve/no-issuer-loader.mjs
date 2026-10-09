export default function () {
	throw new Error(`txt without issuer: ${this.resourcePath}`);
}

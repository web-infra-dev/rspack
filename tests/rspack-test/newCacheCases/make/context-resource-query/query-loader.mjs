export default function () {
	return `module.exports = ${JSON.stringify({ query: this.resourceQuery, resource: this.resource })};`;
}

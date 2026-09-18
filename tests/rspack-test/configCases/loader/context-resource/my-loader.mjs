export default function (content) {
	return (
		"module.exports = " +
		JSON.stringify({
			resource: this.resource,
			prev: content
		})
	);
};

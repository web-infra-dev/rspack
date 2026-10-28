export default function (content) {
	return (
		"module.exports = " +
		JSON.stringify({
			resourceFragment: this.resourceFragment,
			prev: content
		})
	);
};

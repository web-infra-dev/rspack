export default function (content) {
	return content.replace("__SOURCEMAP__", this.sourceMap);
};

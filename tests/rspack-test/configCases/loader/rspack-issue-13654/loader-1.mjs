import path from "node:path";

export default function (content, sourceMap) {
	this.callback(null, content, sourceMap, {
		fromLoader1: path.basename(this.resourcePath)
	});
};

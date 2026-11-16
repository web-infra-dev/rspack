export default function loader(content) {
	this._compilation.times ??= 0;
	const times = ++this._compilation.times;
	return content.replace("1", times);
};

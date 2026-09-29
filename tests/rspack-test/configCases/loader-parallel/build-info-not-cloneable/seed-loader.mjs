export default function (content) {
	this._module.buildInfo.onDone = () => "not cloneable";
	return content;
};

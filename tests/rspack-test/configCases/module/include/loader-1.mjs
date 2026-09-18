export default function (content) {
	this.callback(null, content.replace("42", "43"));
};

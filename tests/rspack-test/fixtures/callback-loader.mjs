export default function(content) {
	var cb = this.getOptions().callback;
	cb(this);
	return content
}

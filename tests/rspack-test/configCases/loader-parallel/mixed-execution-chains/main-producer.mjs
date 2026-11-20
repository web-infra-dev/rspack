export default function (content) {
	this.callback(null, `${content.trim()}:from-main`, null, {
		fromMain: 'ok',
		count: 42,
	});
}

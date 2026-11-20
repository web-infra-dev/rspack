export function pitch() {
	this.callback(null, 'from-main-pitch', null, {
		fromPitch: true,
		count: 7,
	});
}

export default function () {
	throw new Error('main-pitch-producer normal stage should not run');
}

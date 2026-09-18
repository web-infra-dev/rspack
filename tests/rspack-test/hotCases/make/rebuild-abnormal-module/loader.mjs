let times = 0;

export default async function (code) {
	times++;
	if (times === 2) {
		return ")))";
	}
	this.cacheable(false);
	return code.replace("<times>", times);
};

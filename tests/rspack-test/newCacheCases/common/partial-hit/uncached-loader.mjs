let builds = 0;

export default function () {
	this.cacheable(false);
	return `export default ${++builds}`;
};

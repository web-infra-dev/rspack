export function load(name) {
	return import(
		/* webpackMode: "sync", webpackChunkName: "all-modules" */ `./lazy-once/${name}.js`
	);
}

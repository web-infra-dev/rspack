export function load(name) {
	return import(
		/* webpackMode: "invalid", webpackChunkName: "per-module" */ `./lazy/${name}.js`
	);
}

export function load(name) {
	return import(/* webpackMode: "eager", webpackInclude: /allowed/, webpackExclude: /denied/ */ `./allowed/${name}.js`);
}

import(
	/* webpackChunkName: "foo" */
	/* webpackExports: "default" */
	"./module"
).then(module => {
	console.log(module.default);
});
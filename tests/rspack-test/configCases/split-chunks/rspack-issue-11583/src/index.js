import(
	/* webpackChunkName: "foo" */
	"./module"
).then(module => {
	console.log(module.default);
});
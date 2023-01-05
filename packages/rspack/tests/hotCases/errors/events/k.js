import i from "./i";
export default i;

if (module.hot) {
	module.hot.accept(
		"./i",
		() => {
			// TODO remove this
			__webpack_require__("./i.js");
		},
		(err, { moduleId, dependencyId }) => {
			debugger
			throw new Error(
				`Error in accept error handler: ${moduleId} -> ${dependencyId}`
			);
		}
	);
}

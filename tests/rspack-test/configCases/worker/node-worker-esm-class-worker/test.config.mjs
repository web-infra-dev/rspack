export default {
	findBundle() {
		return "./bundle.mjs";
	},
	moduleScope(scope) {
		scope.URL = URL;
	}
};

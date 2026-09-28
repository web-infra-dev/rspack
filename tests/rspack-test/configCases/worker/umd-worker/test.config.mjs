export default {
	moduleScope(scope) {
		delete scope.document.baseURI;
	}
};

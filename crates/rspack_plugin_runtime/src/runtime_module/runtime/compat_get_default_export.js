// getDefaultExport function for compatibility with non-harmony modules
__webpack_require__.n = function(module) {
	var getter = module && module.__esModule ?
        function() { return module['default']; } :
        function() { return module; }
	__webpack_require__.d(getter, { a: getter });
	return getter;
};
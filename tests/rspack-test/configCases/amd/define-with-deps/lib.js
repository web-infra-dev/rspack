define(['./add', './constants', './hello'], function (add, constants, hello) {

	return {
		add,
		FOO: constants.FOO,
		hello,
	};
});

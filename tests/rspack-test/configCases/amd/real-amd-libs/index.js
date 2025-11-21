
it('should work with jquery', function () {
	const $ = require('./jquery');
	expect(typeof $).toBe('function');
	expect($()).toBe('hi jQuery');
	expect($.version).toBe('3.7.1');
});

it('should work with jquery-ui', function () {
	const ui = require('./jquery-ui');
	expect(ui).toStrictEqual({ version: '0.0.0' });
});

it('should work with json-logic-js', function () {
	const jsonLogic = require('./json-logic-js');
	expect(jsonLogic).toStrictEqual({ version: '0.0.0' });
});

import webpackUmdOutput from './webpack-umd-output';
it('should work with webpack umd output', function () {
	expect(webpackUmdOutput).toStrictEqual({ version: '0.0.0' });
});

it('should work with define-in-params', function () {
	const lib = require('./define-in-params');
	expect(lib).toBe(1);
});

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

it("should able to require the css module as commonjs", () => {
	const style = require("./style.module.css");
	console.log(style)
	const interoperatedStyle = _interopRequireDefault(require("./style.module.css"));

	expect(style).toEqual({ foo: '-__style_module_css-foo' });
	expect(style).not.toEqual(nsObj({ foo: '-__style_module_css-foo' }));
	expect(style.__esModule).toEqual(undefined);
	expect(interoperatedStyle.default.foo).toEqual("-__style_module_css-foo");
});

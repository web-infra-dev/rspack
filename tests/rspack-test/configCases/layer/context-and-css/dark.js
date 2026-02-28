require.context('./test1', true, /\.less$/);
require('./test2/shared.less');

it("should contain only black", function () {
	const style = getLinkSheet(document.querySelectorAll("link")[1]);
	expect(style).toContain(`color-dark: black;`);
	expect(style).toContain(`background-dark: black;`);
});

require.context('./test1', true, /\.less$/);
require('./test2/shared.less');

it("should contain only white", function () {
	const style = getLinkSheet(document.querySelectorAll("link")[0]);
	expect(style).toContain(`color-light: white;`);
	expect(style).toContain(`background-light: white;`);
});

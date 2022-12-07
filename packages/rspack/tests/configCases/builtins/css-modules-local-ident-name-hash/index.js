it("css modules composes", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		"#": "1148a7 ",
		"##": "ce53c4 ",
		"#.#.#": "b02409 ",
		"#fake-id": "db28d9 ",
		"++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.":
			"3bc556 ",
		"-a-b-c-": "3d88c9 ",
		"-a0-34a___f": "596555 ",
		".": "d573f2 ",
		123: "1b3826 ",
		"1a2b3c": "c00e5d ",
		":)": "29c1d7 ",
		":`(": "9f9aa5 ",
		":hover": "8a428f ",
		":hover:focus:active": "7803a3 ",
		"<><<<>><>": "6c2a86 ",
		"<p>": "980d83 ",
		"?": "4ad205 ",
		"@": "683720 ",
		"B&W?": "376434 ",
		"[attr=value]": "aff5b7 ",
		_: "5e16c0 ",
		_test: "816b24 ",
		className: "b74ae3 ",
		"f!o!o": "82532b ",
		"f'o'o": "9ec6b9 ",
		"f*o*o": "87a42e ",
		"f+o+o": "56826a ",
		"f/o/o": "37b7b8 ",
		"f\\o\\o": "15fc51 ",
		"foo.bar": "588963 ",
		"foo/bar": "3aa117 ",
		"foo/bar/baz": "5c97a9 ",
		"foo\\bar": "647d2b ",
		"foo\\bar\\baz": "4d3ac0 ",
		"f~o~o": "8c76aa ",
		"m_x_@": "878e44 ",
		someId: "2423e5 ",
		subClass: "a3f399 ",
		test: "1c639c ",
		"{}": "7d10e4 ",
		"©": "f53585 ",
		"“‘’”": "4c3b81 ",
		"⌘⌥": "685f93 ",
		"☺☃": "445077 ",
		"♥": "a47a28 ",
		"𝄞♪♩♫♬": "e79593 ",
		"💩": "a3a765 ",
		"😍": "3ffc35 "
	});
});

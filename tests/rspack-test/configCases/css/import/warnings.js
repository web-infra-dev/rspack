"use strict";

module.exports = [
	/Expected URL in '@import nourl\(test\.css\);'/,
	/Expected URL in '@import ;'/,
	/Expected URL in '@import foo-bar;'/,
	/An URL in 'layer\(super\.foo\) "\.\/style2\.css\?warning=1"' should be before 'layer\(\.\.\.\)' or 'supports\(\.\.\.\)'/,
	/An URL in 'layer\(super\.foo\) supports\(display: flex\) "\.\/style2\.css\?warning=2"' should be before 'layer\(\.\.\.\)' or 'supports\(\.\.\.\)'/,
	/An URL in 'layer\(super\.foo\) supports\(display: flex\) screen and \(min-width: 400px\) "\.\/style2\.css\?warning=3"' should be before 'layer\(\.\.\.\)' or 'supports\(\.\.\.\)'/,
	/An URL in 'layer\(super\.foo\) url\("\.\/style2\.css\?warning=4"\)' should be before 'layer\(\.\.\.\)' or 'supports\(\.\.\.\)'/,
	/An URL in 'layer\(super\.foo\) supports\(display: flex\) url\("\.\/style2\.css\?warning=5"\)' should be before 'layer\(\.\.\.\)' or 'supports\(\.\.\.\)'/,
	/An URL in 'layer\(super\.foo\) supports\(display: flex\) screen and \(min-width: 400px\) url\("\.\/style2\.css\?warning=6"\)' should be before 'layer\(\.\.\.\)' or 'supports\(\.\.\.\)'/,
	/The 'layer\(\.\.\.\)' in 'supports\(display: flex\) layer\(super\.foo\)' should be before 'supports\(\.\.\.\)'/,
	/'@namespace' is not supported in bundled CSS/,
	/Expected URL in '@import layer\(test\) supports\(background: url\("\.\/img\.png"\)\) screen and \(min-width: 400px\);'/,
	/Expected URL in '@import screen and \(min-width: 400px\);'/,
	/Expected URL in '@import supports\(background: url\("\.\/img\.png"\)\) screen and \(min-width: 400px\);'/,
	/Expected URL in '@import supports\(background: url\("\.\/img\.png"\)\);'/,
	/Duplicate of 'url\(\.\.\.\)' in '@import url\(\.\/style2\.css\?multiple=1\) url\(\.\/style2\.css\?multiple=2\)'/,
	/Duplicate of 'url\(\.\.\.\)' in '@import url\("\.\/style2\.css\?multiple=3"\) url\("\.\/style2\.css\?multiple=4"/,
	/Duplicate of 'url\(\.\.\.\)' in '@import "\.\/style2\.css\?strange=3" url\("\.\/style2\.css\?multiple=4"/
];

"use strict";

// The fixture mocks an unresolvable package on purpose (asserting `r: null`);
// the optional mock dependency downgrades the miss to this warning. (The
// 1-arg auto-mock's missing `__mocks__` file warns nothing — it falls back to
// `{ mock: true }` by design.)
module.exports = [
	/Can't resolve 'missing-pkg-for-resolved-info'/,
];

function createSet() {
	return new Set();
}

export const marker = 1;

let directNew = /*#__PURE__*/ new Set();
let wrappedNew = /*#__PURE__*/ (new Set());
let wrappedCall = /*#__PURE__*/ (createSet());
let extraTextNew = /* #__PURE__ additional text */ new Set();
let atPureNew = /* @__PURE__ additional text */ new Set();
let notPureComment = /* note: keep webpack-compatible #__PURE__ syntax here */ createSet();

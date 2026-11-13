export function read() {
	return [__webpack_public_path__, __webpack_nonce__];
}

export function write(publicPath, nonce) {
	__webpack_public_path__ = publicPath;
	__webpack_nonce__ = nonce;
}

import cssLoader from "css-loader";

export default function cssProxyLoader(code) {
	cssLoader.call(this, code)
}

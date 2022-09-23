import React from "react";
import ReactDOM from "react-dom";
import "./base.css";
import LogoJPG from "./file.jpg";
import LogoPNG from "./file.png";
import LogoSVG from "./file.svg";
// import Dark from './dark.svg';
// import Light from './light.svg'
// import LogoUrl from './logo.svg'
// import Logo from './logo.svg'
// const Button = React.lazy(() => import('../src/button'))
// console.log('LogoUrl', LogoUrl)
// console.log('Logo', Logo)
const App = () => {
	return React.createElement(
		React.Suspense,
		{ fallback: React.createElement("div", null, "loading...") },
		React.createElement("div", null, "hello world"),
		React.createElement("img", {
			style: { width: "40px", height: "40px" },
			src: LogoJPG,
			alt: "logo jpg"
		}),
		React.createElement("img", {
			style: { width: "40px", height: "40px" },
			src: LogoPNG,
			alt: "logo png"
		}),
		React.createElement("img", {
			style: { width: "40px", height: "40px" },
			src: LogoSVG,
			alt: "logo svg"
		})
	);
};
ReactDOM.render(
	React.createElement(App, null),
	document.getElementById("root")
);

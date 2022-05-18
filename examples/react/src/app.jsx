import "./base.css";
import React from "react";
import ReactDOM from "react-dom";
import LogoUrl from "./logo.svg?raw";
import Logo from "./logo.svg";
const Button = React.lazy(() => import("../src/button"));

console.log("LogoUrl", LogoUrl);
console.log("Logo", Logo);
const App = () => {
  return (
    <React.Suspense fallback={<div>loading...</div>}>
      <div>hello world</div>
      <Button></Button>

      <img src={LogoUrl} alt="logo" />
      <Logo />
      <Logo width={"20em"} height={"20em"} />
    </React.Suspense>
  );
};
ReactDOM.render(<App />, document.getElementById("root"));

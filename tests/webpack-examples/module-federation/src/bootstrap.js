import React from "react"; // <- this is a shared module, but used as usual
import ReactDom from "react-dom";
import App from "./App";

// load app
const el = document.createElement("main");
ReactDom.render(<App />, el);
document.body.appendChild(el);

// remove spinner
document.body.removeChild(document.getElementsByClassName("spinner")[0]);

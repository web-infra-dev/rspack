import React from "react";
import ReactDOM from "react-dom";
const Button = React.lazy(import("./button"));

const App = () => {
	<div>
    <div>app</div>
    <Button></Button>
  </div>;
};

ReactDOM.render(<App />, document.getElementById("root"));

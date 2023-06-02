import React from "react";
import "./index.css";
import { ContextComponent  } from "./CountProvider";
import { ReactRefreshFinder } from './ReactRefreshFinder';

const Button = () => {
	const [count, setCount] = React.useState(10);
    return <button onClick={() => setCount(count => count + 1)}>{count}</button>
}

export const App = () => {
	return (
		<div className="App">
      	<div className="header">Hello World</div>
			<Button />
			<div className="placeholder">__PLACE_HOLDER__</div>
			<ContextComponent/>
			<ReactRefreshFinder/>
		</div>
	);
};
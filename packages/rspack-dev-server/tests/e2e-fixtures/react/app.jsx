import React from "react";
import "./index.css";

export const App = () => {
	const [count, setCount] = React.useState(0);
	return (
		<div>
			<div className="test-button" onClick={() => setCount(() => count + 1)}>
				<span className="test-button-content">{count}</span>
			</div>
			<div className="placeholder">__PLACE_HOLDER__</div>
		</div>
	);
};

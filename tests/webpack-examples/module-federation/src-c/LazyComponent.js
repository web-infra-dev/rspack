import random from "lodash/random";
import React from "react";

const Component = () => (
	<div style={{ border: "5px solid darkgreen" }}>
		<p>I'm a lazy Component exposed from container C!</p>
		<p>I'm lazy loaded by the app and lazy load another component myself.</p>
		<p>Using lodash in Remote: {random(0, 6)}</p>
	</div>
);
export default Component;

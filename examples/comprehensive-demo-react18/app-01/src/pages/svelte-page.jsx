import React from "react";
import Page from "../Page";
import { TextField, Box } from "@mui/material";
import loadApp from "app_04/loadApp";

const SveltePage = () => {
	const [name, setName] = React.useState("federation");
	const mountEl = React.useRef();

	React.useEffect(() => {
		if (mountEl.current.innerHTML.length === 0) {
			loadApp("app_04", name);
		}
	});

	const handleChange = e => {
		setName(e.target.value);
		const event = new CustomEvent("change-name", {
			detail: {
				name: e.target.value
			},
			bubbles: true,
			cancelable: true,
			composed: true // makes the event jump shadow DOM boundary
		});
		let source = e.target || e.srcElement;
		source.dispatchEvent(event);
	};

	return (
		<Page title="Svelte Demo">
			<Box
				component="form"
				noValidate
				autoComplete="off"
				sx={{
					"& > *": {
						margin: 1,
						width: 200
					}
				}}
			>
				<TextField
					id="standard-basic"
					label="Name"
					value={name}
					onChange={e => handleChange(e)}
				/>
				<div id="app_04" ref={mountEl}></div>
			</Box>
		</Page>
	);
};

export default SveltePage;

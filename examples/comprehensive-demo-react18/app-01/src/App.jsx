import { CssBaseline, Box } from "@mui/material";

import { HashRouter } from "react-router-dom";
import React from "react";
import Routes from "./Routes";
import SideNav from "./SideNav";

function App() {
	return (
		<HashRouter>
			<CssBaseline />
			<Box sx={{ display: "flex" }}>
				<SideNav />
				<Routes />
			</Box>
		</HashRouter>
	);
}

export default App;

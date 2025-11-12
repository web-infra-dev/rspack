import { AppBar, Box, Tab, Tabs, Typography } from "@mui/material";
import {
	Redirect,
	Route,
	Switch,
	useHistory,
	useLocation,
	useRouteMatch
} from "react-router-dom";

import React from "react";

const Button = React.lazy(() => import("app_03/Button"));

export default function TabsComponent() {
	const match = useRouteMatch();
	const history = useHistory();
	const location = useLocation();
	const { path: rootPath } = match;

	const handleChange = (event, newValue) => {
		history.push(newValue);
	};

	return (
		<Box
			sx={{
				flexGrow: 1,
				backgroundColor: theme => theme.palette.background.paper
			}}
		>
			<AppBar position="static">
				<Tabs value={location.pathname} onChange={handleChange}>
					<Tab label="Foo" value={`${rootPath}/foo`} />
					<Tab label="Bar" value={`${rootPath}/bar`} />
				</Tabs>
			</AppBar>
			<Switch>
				<Route path={rootPath} exact={true}>
					<Redirect to={`${rootPath}/foo`} />
				</Route>
				<Route path={`${rootPath}/foo`}>
					<Typography component="div">
						<Box p={3}>Foo Content</Box>
					</Typography>
				</Route>
				<Route path={`${rootPath}/bar`}>
					<Typography component="div">
						<Box p={3}>
							Bar Content
							<React.Suspense fallback={null}>
								<Button>Bar Button</Button>
							</React.Suspense>
						</Box>
					</Typography>
				</Route>
			</Switch>
		</Box>
	);
}

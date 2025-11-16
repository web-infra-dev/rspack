import {
	Divider,
	Drawer,
	List,
	ListItem,
	ListItemText,
	ListSubheader,
	Typography,
	Box
} from "@mui/material";

import { Link } from "react-router-dom";
import React from "react";

const drawerWidth = 240;

export default function SideNav() {
	return (
		<Drawer
			sx={{
				width: drawerWidth,
				flexShrink: 0,
				"& .MuiDrawer-paper": {
					width: drawerWidth
				}
			}}
			variant="permanent"
			anchor="left"
		>
			<Box
				sx={{
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					minHeight: 64
				}}
			>
				<Typography variant="h6">SideNav</Typography>
			</Box>
			<Divider />
			<List>
				<ListSubheader>Demo Pages</ListSubheader>
				<ListItem button component={Link} to="/">
					<ListItemText primary="Main" />
				</ListItem>
				<ListItem button component={Link} to="/ui-library">
					<ListItemText primary="UI Library" />
				</ListItem>
				<ListItem button component={Link} to="/dialog">
					<ListItemText primary="Dialog" />
				</ListItem>
				<ListItem button component={Link} to="/svelte">
					<ListItemText primary="Svelte Page" />
				</ListItem>
				<ListItem button component={Link} to="/routing/foo">
					<ListItemText primary="Routing" />
				</ListItem>
				<ListSubheader>Apps</ListSubheader>
				<ListItem button component="a" href="http://localhost:3001">
					<ListItemText primary="App #1" secondary="http://localhost:3001" />
				</ListItem>
				<ListItem button component="a" href="http://localhost:3002">
					<ListItemText primary="App #2" secondary="http://localhost:3002" />
				</ListItem>
				<ListItem button component="a" href="http://localhost:3003">
					<ListItemText primary="App #3" secondary="http://localhost:3003" />
				</ListItem>
				<ListItem button component="a" href="http://localhost:3004">
					<ListItemText primary="App #4" secondary="http://localhost:3004" />
				</ListItem>
			</List>
		</Drawer>
	);
}

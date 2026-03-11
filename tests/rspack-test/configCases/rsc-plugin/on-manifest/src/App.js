"use server-entry";

import { Client } from './Client';
import './App.css';

export const App = () => {
	return (
		<>
			<h1>RSC App</h1>
			<Client />
		</>
	);
};

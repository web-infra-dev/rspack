"use server-entry";

import { Page1Child } from './Page1Child';
import './Page1.css';

export const Page1 = async () => {
	return (
		<section className="page-one-css">
			<h2>Page 1</h2>
			<Page1Child />
		</section>
	);
};

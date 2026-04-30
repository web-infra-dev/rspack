"use server-entry";

import { SharedServerComponent } from '../SharedServerComponent';
import { PageBChild } from './PageBChild';
import './PageB.css';

export const PageB = async () => {
	return (
		<section className="page-b-async-chunks-false-css">
			<h2>Page B</h2>
			<SharedServerComponent page="Page B" />
			<PageBChild />
		</section>
	);
};

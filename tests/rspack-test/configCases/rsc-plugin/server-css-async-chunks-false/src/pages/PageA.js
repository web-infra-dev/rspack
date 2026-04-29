"use server-entry";

import { SharedServerComponent } from '../SharedServerComponent';
import { PageAChild } from './PageAChild';
import { PageANested } from './PageANested';
import './PageA.css';

export const PageA = async () => {
	return (
		<section className="page-a-async-chunks-false-css">
			<h2>Page A</h2>
			<SharedServerComponent page="Page A" />
			<PageAChild />
			<PageANested />
		</section>
	);
};

"use server-entry";

import { Page1Child } from './Page1Child';
import './Page1.css';
import '../Shared.css';

export const Page1 = async () => {
	return (
		<section className="page-one-css">
			<h2>Page 1</h2>
			<p className="shared-server-css">Shared stylesheet from Page 1</p>
			<p className="shared-nested-server-css">
				Shared nested stylesheet from Page 1
			</p>
			<Page1Child />
		</section>
	);
};

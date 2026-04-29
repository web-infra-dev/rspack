"use server-entry";

import { Page2Child } from './Page2Child';
import './Page2.css';

export const Page2 = async () => {
	return (
		<section className="page-two-css">
			<h2>Page 2</h2>
			<p className="shared-server-css">Shared stylesheet from Page 2</p>
			<p className="shared-nested-server-css">
				Shared nested stylesheet from Page 2
			</p>
			<Page2Child />
		</section>
	);
};

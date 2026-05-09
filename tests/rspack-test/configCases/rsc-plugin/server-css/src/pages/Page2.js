"use server-entry";

import { Page2Child } from './Page2Child';
import { SharedServerComponent } from '../SharedServerComponent';
import './Page2.css';

export const Page2 = async () => {
	return (
		<section className="page-two-css">
			<h2>Page 2</h2>
			<p className="shared-server-css">Shared stylesheet from Page 2</p>
			<p className="shared-nested-server-css">
				Shared nested stylesheet from Page 2
			</p>
			<SharedServerComponent page="Page 2" />
			<Page2Child />
		</section>
	);
};

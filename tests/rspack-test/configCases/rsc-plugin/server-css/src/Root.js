import { Page1 } from './pages/Page1';
import { Page2 } from './pages/Page2';
// Keep the page imports before Shared.css to exercise shared CSS ownership:
// Page1 reaches Shared.css through a server-entry chain first, then this root
// import must still make the same stylesheet root-owned.
import './Root.css';
import './Shared.css';

export const Root = async () => {
	return (
		<main className="root-server-css">
			<h1>RSC Root</h1>
			<p className="shared-server-css">Shared stylesheet from root</p>
			<p className="shared-nested-server-css">
				Shared nested stylesheet from root
			</p>
			<Page1 />
			<Page2 />
		</main>
	);
};

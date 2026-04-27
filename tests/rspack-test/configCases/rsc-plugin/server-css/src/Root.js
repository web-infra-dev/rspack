import { Page1 } from './pages/Page1';
import { Page2 } from './pages/Page2';
import './Root.css';

export const Root = async () => {
	return (
		<main className="root-server-css">
			<h1>RSC Root</h1>
			<Page1 />
			<Page2 />
		</main>
	);
};

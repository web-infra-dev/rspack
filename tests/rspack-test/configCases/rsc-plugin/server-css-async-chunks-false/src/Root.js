import { PageA } from './pages/PageA';
import { PageB } from './pages/PageB';
import './Root.css';

export const Root = async () => {
	return (
		<main className="root-async-chunks-false-css">
			<h1>RSC Root</h1>
			<PageA />
			<PageB />
		</main>
	);
};

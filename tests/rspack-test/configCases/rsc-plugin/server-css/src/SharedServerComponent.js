import './ServerEntryShared.css';

export const SharedServerComponent = ({ page }) => {
	return (
		<p className="server-entry-shared-css">
			Shared server component stylesheet from {page}
		</p>
	);
};

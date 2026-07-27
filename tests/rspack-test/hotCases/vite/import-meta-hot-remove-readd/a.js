const data = import.meta.hot.data;
data.marker = "initial";

if (import.meta.hot) {
	import.meta.hot.dispose(disposedData => {
		disposedData.disposed = true;
	});
}

export default data;

---

---

export default import.meta.hot.data;

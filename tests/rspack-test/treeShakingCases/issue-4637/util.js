import { _ } from "@swc/helpers/_/_create_class";
var ConsoleExporterWeb;
ConsoleExporterWeb = (function () {
	"use strict";
	function ConsoleExporterWeb() {
		this.stoped = false;
	}
	_(ConsoleExporterWeb, [
		{
			key: "export",
			value: function _export(evts, cb) {
				if (this.stoped) return;
				evts.forEach(adaptToBrowserConsole);
				if (cb) cb(ExportResult.SUCCESS);
			}
		},
		{
			key: "shutdown",
			value: function shutdown() {
				this.stoped = true;
			}
		}
	]);
	return ConsoleExporterWeb;
})();

export default ConsoleExporterWeb;

"use strict";

module.exports = async function sessionSubscribe(session) {
	session.on("sessionattached", s => {
		sessionSubscribe(s);
	});
	session.send("Network.enable");
	session.send("Runtime.runIfWaitingForDebugger");
};

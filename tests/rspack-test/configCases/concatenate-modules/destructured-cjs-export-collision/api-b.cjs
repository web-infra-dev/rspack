class Command {
	constructor(name) {
		this.name = `b:${name}`;
	}
}

function createCommand(name) {
	return new Command(name);
}

exports.createCommand = createCommand;
exports.Command = Command;

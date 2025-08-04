// Shared component library - testing various export scenarios

// Used export (imported in index.js)
export class Button {
	constructor(text, onClick) {
		this.element = document.createElement("button");
		this.element.textContent = text;
		this.element.addEventListener("click", onClick);
	}

	render() {
		return this.element;
	}
}

// Used export (imported in index.js)
export class Modal {
	constructor(title, content) {
		this.title = title;
		this.content = content;
		this.isOpen = false;
	}

	open() {
		this.isOpen = true;
		console.log(`Modal "${this.title}" opened`);
	}

	close() {
		this.isOpen = false;
		console.log(`Modal "${this.title}" closed`);
	}
}

// Unused export (not imported anywhere)
export const createCard = (title, description) => {
	return {
		title,
		description,
		render() {
			return `<div class="card"><h3>${title}</h3><p>${description}</p></div>`;
		}
	};
};

// Additional unused exports for testing
export class Tooltip {
	constructor(element, text) {
		this.element = element;
		this.text = text;
	}

	show() {
		console.log(`Showing tooltip: ${this.text}`);
	}
}

export const createAlert = (message, type = "info") => {
	return {
		message,
		type,
		show() {
			console.log(`Alert (${type}): ${message}`);
		}
	};
};

// Default export (not imported but defined)
export default {
	Button,
	Modal,
	createCard,
	Tooltip,
	createAlert
};

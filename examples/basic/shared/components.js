// Shared component library
export class Button {
  constructor(text, onClick) {
    this.element = document.createElement('button');
    this.element.textContent = text;
    this.element.addEventListener('click', onClick);
  }

  render() {
    return this.element;
  }
}

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

export const createCard = (title, description) => {
  return {
    title,
    description,
    render() {
      return `<div class="card"><h3>${title}</h3><p>${description}</p></div>`;
    }
  };
};

export default {
  Button,
  Modal,
  createCard
};
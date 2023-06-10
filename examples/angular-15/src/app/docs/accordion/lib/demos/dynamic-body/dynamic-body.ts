import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-accordion-dynamic-body',
  templateUrl: './dynamic-body.html'
})
export class DemoAccordionDynamicBodyComponent {
  items = ['Item 1', 'Item 2', 'Item 3'];

  addItem(): void {
    this.items.push(`Item ${this.items.length + 1}`);
  }

  removeItem(): void {
    this.items = this.items.slice(0, this.items.length - 1);
  }
}

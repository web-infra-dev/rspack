import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-accordion-dynamic',
  templateUrl: './dynamic.html'
})
export class DemoAccordionDynamicComponent {
  groups = [
    {
      title: 'Dynamic Group Header - 1',
      content: 'Dynamic Group Body - 1'
    },
    {
      title: 'Dynamic Group Header - 2',
      content: 'Dynamic Group Body - 2'
    }
  ];

  addGroupItem(): void {
    this.groups.push({
      title: `Dynamic Group Header - ${this.groups.length + 1}`,
      content: `Dynamic Group Body - ${this.groups.length + 1}`
    });
  }
}

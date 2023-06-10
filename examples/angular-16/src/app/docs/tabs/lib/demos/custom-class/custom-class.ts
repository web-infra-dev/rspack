import { Component } from '@angular/core';

interface ITab {
  title: string;
  content: string;
  removable?: boolean;
  disabled?: boolean;
  active?: boolean;
  customClass?: string;
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-tabs-custom-class',
  templateUrl: './custom-class.html'
})
export class DemoTabsCustomClassComponent {
  tabs: ITab[] = [
    {
      title: 'Dynamic Title 1',
      content: 'Dynamic content 1',
      customClass: 'customClass'
    },
    {
      title: 'Dynamic Title 2',
      content: 'Dynamic content 2',
      customClass: 'customClass'
    }
  ];
}

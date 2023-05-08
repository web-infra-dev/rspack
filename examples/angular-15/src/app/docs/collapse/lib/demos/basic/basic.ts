import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'collapse-demo',
  templateUrl: './basic.html'
})
export class CollapseDemoComponent {
  isCollapsed = false;
}

import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-popover-trigger-by-isopen',
  templateUrl: './trigger-by-isopen-property.html'
})
export class DemoPopoverByIsOpenPropComponent {
  isOpen = false;
}

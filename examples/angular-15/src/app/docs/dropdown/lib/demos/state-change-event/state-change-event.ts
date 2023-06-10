import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-dropdown-state-change-event',
  templateUrl: './state-change-event.html'
})
export class DemoDropdownStateChangeEventComponent {
  text?: string;
  onOpenChange(data: boolean): void {
    this.text = data ? 'opened' : 'closed';
  }
}

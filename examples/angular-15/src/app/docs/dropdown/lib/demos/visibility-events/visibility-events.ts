import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-dropdown-visibility-events',
  templateUrl: './visibility-events.html'
})
export class DemoDropdownVisibilityEventsComponent {
  messages: string[] = [];
  message = 'onShown';

  handler(value: string): void {
    this.messages.push(`Event ${value} is fired`);
    this.messages = this.messages.length > 2 ? this.messages.slice(0, 1) : this.messages;
  }
}

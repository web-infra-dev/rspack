import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-visibility-events',
  templateUrl: './visibility-events.html'
})

export class DemoDatePickerVisibilityEventsComponent {
  messages: string[] = [];
  message = 'onShown';
  handler(value: string): void {
    if(this.message === value) {
      this.messages = [];
    }
    this.messages.push(`Event ${value} is fired`);
  }
}

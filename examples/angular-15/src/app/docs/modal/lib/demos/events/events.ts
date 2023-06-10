import { Component, ViewChild } from '@angular/core';
import { ModalDirective } from 'ngx-bootstrap/modal';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-modal-events',
  templateUrl: './events.html',
  styles: [`
    .card {
      margin-bottom: 0.75rem;
      padding: 8px;
    }
  `]
})
export class DemoModalEventsComponent {
  @ViewChild(ModalDirective, { static: false }) modal?: ModalDirective;
  messages?: string[];

  showModal() {
    this.messages = [];
    this.modal?.show();
  }
  handler(type: string, $event: ModalDirective) {
    this.messages?.push(
      `event ${type} is fired${$event.dismissReason
        ? ', dismissed by ' + $event.dismissReason
        : ''}`
    );
  }
}

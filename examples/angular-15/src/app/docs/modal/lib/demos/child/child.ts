import { Component, ViewChild } from '@angular/core';
import { ModalDirective } from 'ngx-bootstrap/modal';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-modal-child',
  templateUrl: './child.html'
})
export class DemoModalChildComponent {
  @ViewChild('childModal', { static: false }) childModal?: ModalDirective;

  showChildModal(): void {
    this.childModal?.show();
  }

  hideChildModal(): void {
    this.childModal?.hide();
  }
}

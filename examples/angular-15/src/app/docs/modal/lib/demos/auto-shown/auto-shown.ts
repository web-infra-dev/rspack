import { Component, ViewChild } from '@angular/core';
import { ModalDirective } from 'ngx-bootstrap/modal';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-modal-auto-shown',
  templateUrl: './auto-shown.html'
})
export class DemoAutoShownModalComponent {
  @ViewChild('autoShownModal', { static: false }) autoShownModal?: ModalDirective;
  isModalShown = false;

  showModal(): void {
    this.isModalShown = true;
  }

  hideModal(): void {
    this.autoShownModal?.hide();
  }

  onHidden(): void {
    this.isModalShown = false;
  }
}

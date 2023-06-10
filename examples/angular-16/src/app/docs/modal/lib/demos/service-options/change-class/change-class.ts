import { Component, TemplateRef } from '@angular/core';
import { BsModalService, BsModalRef } from 'ngx-bootstrap/modal';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-modal-change-class',
  templateUrl: './change-class.html'
})
export class DemoModalServiceChangeClassComponent {
  modalRef?: BsModalRef;
  valueWidth = false;
  constructor(private modalService: BsModalService) {}

  openModal(template: TemplateRef<any>) {
    this.modalRef = this.modalService.show(
      template,
      Object.assign({}, { class: 'modal-sm' })
    );
  }

  setModalClass() {
    this.valueWidth = !this.valueWidth;
    const modalWidth = this.valueWidth ? 'modal-lg' : 'modal-sm';
    this.modalRef?.setClass(modalWidth);
  }
}

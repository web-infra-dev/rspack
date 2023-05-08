import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-carousel-multilist-indicators-by-chunk',
  templateUrl: './multilist-indicators-by-chunk.html'
})
export class DemoCarouselMultilistIndicatorsByChunkComponent {
  itemsPerSlide = 3;

  slides = [
    {image: 'assets/images/nature/1.jpg'},
    {image: 'assets/images/nature/2.jpg'},
    {image: 'assets/images/nature/3.jpg'},
    {image: 'assets/images/nature/4.jpg'},
    {image: 'assets/images/nature/5.jpg'},
    {image: 'assets/images/nature/6.jpg'},
    {image: 'assets/images/nature/7.jpg'},
    {image: 'assets/images/nature/8.jpg'},
    {image: 'assets/images/nature/1.jpg'},
    {image: 'assets/images/nature/2.jpg'}
  ];
}

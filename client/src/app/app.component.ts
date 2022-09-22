import { HttpClient } from '@angular/common/http';
import { AfterViewInit, Component, ElementRef, ViewChild } from '@angular/core';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements AfterViewInit {
  title = 'client';

  @ViewChild("mainCanvas") mainCanvas?: ElementRef<HTMLCanvasElement>;

  constructor(private http: HttpClient) {

  }

  async ngAfterViewInit() {

  }
}



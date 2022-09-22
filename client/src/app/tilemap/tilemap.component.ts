import { HttpClient } from '@angular/common/http';
import {
  AfterViewInit,
  Component,
  ElementRef,
  OnInit,
  ViewChild,
} from '@angular/core';

@Component({
  selector: 'app-tilemap',
  templateUrl: './tilemap.component.html',
  styleUrls: ['./tilemap.component.scss'],
})
export class TilemapComponent implements OnInit, AfterViewInit {
  @ViewChild('mainCanvas') mainCanvas?: ElementRef<HTMLCanvasElement>;

  constructor(private http: HttpClient) {}

  ngOnInit(): void {}

  ngAfterViewInit() {
    this.http
      .get<[number[][], string[][]]>(
        'http://localhost:4200/assets/generated/map.json'
      )
      .subscribe((json) => {
        const data = json[0];
        const classData = json[1];
        const canvas = this.mainCanvas?.nativeElement;
        if (!canvas) return;

        canvas.width = 2048;
        canvas.height = 2048;

        let canvasWidth = canvas.width;
        let canvasHeight = canvas.height;
        let ctx = canvas.getContext('2d');
        if (!ctx) return;

        let canvasData = ctx.getImageData(0, 0, canvasWidth, canvasHeight);

        const drawPixel = (
          x: number,
          y: number,
          r: number,
          g: number,
          b: number,
          a: number
        ) => {
          if (x < 0 || x >= canvasWidth) return;
          if (y < 0 || y >= canvasHeight) return;

          var index = (x + y * canvasWidth) * 4;
          canvasData.data[index + 0] =
            r * (a / 255) + canvasData.data[index + 0] * ((255 - a) / 255);
          canvasData.data[index + 1] =
            g * (a / 255) + canvasData.data[index + 1] * ((255 - a) / 255);
          canvasData.data[index + 2] =
            b * (a / 255) + canvasData.data[index + 2] * ((255 - a) / 255);
          canvasData.data[index + 3] = 255;
        };

        let min = 0;
        let max = Math.max(...data.map((x) => Math.max(...x)));

        let x = 0;
        let y = 0;
        for (let row of data) {
          console.log(row.length);
          for (let height of row) {
            let v = ((height - min) / (max - min)) * ((max - min) / 255);
            drawPixel(x, y, v * 0.95, v * 0.5, v * 0.9, 255);

            if (classData[y][x] != 'EMPTY') {
              drawPixel(x, y, 150, 50, 50, 100);
            }

            if (classData[y][x] == 'WATER') {
              drawPixel(x, y, 150, 50, 200, 100);
            }

            if (classData[y][x] == 'BUILDING') {
              drawPixel(x, y, 255, 180, 230, 255);

              for (let blur_x = -8; blur_x <= 8; ++blur_x) {
                for (let blur_y = -8; blur_y <= 8; ++blur_y) {
                  drawPixel(
                    x,
                    y,
                    255,
                    180,
                    230,
                    255 - Math.min(10 * Math.abs(blur_x * blur_y), 255)
                  );
                }
              }
            }

            x += 1;
          }
          x = 0;
          y += 1;
        }

        ctx.putImageData(canvasData, 0, 0);
      });
  }
}

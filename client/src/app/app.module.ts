import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { HttpClientModule } from '@angular/common/http';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { TilemapComponent } from './tilemap/tilemap.component';
import { WorldComponent } from './world/world.component';

@NgModule({
  declarations: [AppComponent, TilemapComponent, WorldComponent],
  imports: [BrowserModule, AppRoutingModule, HttpClientModule],
  bootstrap: [AppComponent],
})
export class AppModule {}

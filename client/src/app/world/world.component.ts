import { HttpClient } from '@angular/common/http';
import { Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { mat4, vec3, vec4 } from 'gl-matrix';
import { first, interval } from 'rxjs';

const vsSource = `
attribute vec4 aVertexPosition;
attribute vec3 aVertexNormal;
varying highp vec3 FragmentNormal;
uniform mat4 uModelViewMatrix;
uniform mat4 uProjectionMatrix;
uniform mat4 uViewMatrix;

void main() {
  gl_Position = uProjectionMatrix * uViewMatrix * uModelViewMatrix * aVertexPosition;
  // * (2.0 - ((aVertexPosition.z - 1.0) * 2.0))
  FragmentNormal = aVertexNormal;
}
`;

const fsSource = `
varying highp vec3 FragmentNormal;
uniform highp vec3 uColor;
uniform highp vec3 uLightSource;

void main() {
  highp float d = dot(FragmentNormal, uLightSource);
  if (d < 0.5) {
    // d = 0.5;
  }
  if (d > 0.6) {
    // d = 1.0;
  }
  highp float f = 0.1 +  d * 0.9;
  gl_FragColor = vec4(f * uColor, 1.0);
}
`;

function initShaderProgram(
  gl: WebGLRenderingContext,
  vsSource: string,
  fsSource: string
) {
  const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vsSource);
  const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fsSource);
  if (!vertexShader) return null;
  if (!fragmentShader) return null;

  // Create the shader program

  const shaderProgram = gl.createProgram();
  if (!shaderProgram) return null;
  gl.attachShader(shaderProgram, vertexShader);
  gl.attachShader(shaderProgram, fragmentShader);
  gl.linkProgram(shaderProgram);

  // If creating the shader program failed, alert

  if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
    alert(
      `Unable to initialize the shader program: ${gl.getProgramInfoLog(
        shaderProgram
      )}`
    );
    return null;
  }

  return shaderProgram;
}

//
// creates a shader of the given type, uploads the source and
// compiles it.
//
function loadShader(gl: WebGLRenderingContext, type: number, source: string) {
  const shader = gl.createShader(type);
  if (!shader) return null;

  // Send the source to the shader object

  gl.shaderSource(shader, source);

  // Compile the shader program

  gl.compileShader(shader);

  // See if it compiled successfully

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    alert(
      `An error occurred compiling the shaders: ${gl.getShaderInfoLog(shader)}`
    );
    gl.deleteShader(shader);
    return null;
  }

  return shader;
}

function initBuffers(
  gl: WebGLRenderingContext,
  positions: number[],
  normals: number[],
  buildings: number[][],
  buildingNormals: number[][]
) {
  // Create a buffer for the square's positions.

  const positionBuffer = gl.createBuffer();

  // Select the positionBuffer as the one to apply buffer
  // operations to from here out.

  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);

  // Now create an array of positions for the square.

  // Now pass the list of positions into WebGL to build the
  // shape. We do this by creating a Float32Array from the
  // JavaScript array, then use it to fill the current buffer.

  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

  const normalBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, normalBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(normals), gl.STATIC_DRAW);

  const buildingBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buildingBuffer);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array(buildings.flatMap((b) => b)),
    gl.STATIC_DRAW
  );

  const buildingNormalBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buildingNormalBuffer);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array(buildingNormals.flatMap((b) => b)),
    gl.STATIC_DRAW
  );

  return {
    vertexCount: positions.length / 4,
    position: positionBuffer,
    normal: normalBuffer,

    buildings: buildingBuffer,
    buildingNormals: buildingNormalBuffer,
    buildingCount: buildings.length,
  };
}

let rotX = 0;
let y = 0;
let x = 0;
let posX = 0;
let posY = 0;
let posZ = 0;

function createLookingAt() {
  return vec3.fromValues(0 + posX, 10 + posY, -15 + posZ);
}

function createCameraPosition() {
  const lookat = createLookingAt();
  const cameraPosition = vec3.fromValues(0 + posX, 10 + posY, 10 + posZ);
  vec3.rotateX(cameraPosition, cameraPosition, lookat, y);
  vec3.rotateY(cameraPosition, cameraPosition, lookat, x);
  return cameraPosition;
}

function createViewMatrix() {
  const lookat = createLookingAt();
  const cameraPosition = createCameraPosition();
  const viewMatrix = mat4.create();
  mat4.lookAt(viewMatrix, cameraPosition, lookat, [0, 1, 0]);
  return viewMatrix;
}

function drawScene(gl: WebGLRenderingContext, programInfo: any, buffers: any) {
  gl.clearColor(0.1, 0.1, 0.1, 1.0);
  gl.clearDepth(1.0);
  gl.enable(gl.DEPTH_TEST);
  gl.depthFunc(gl.LEQUAL);

  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

  const fieldOfView = (45 * Math.PI) / 180;
  const aspect = gl.canvas.clientWidth / gl.canvas.clientHeight;
  const zNear = 0.1;
  const zFar = 100.0;
  const projectionMatrix = mat4.create();

  mat4.perspective(projectionMatrix, fieldOfView, aspect, zNear, zFar);

  let modelViewMatrix = mat4.create();

  // mat4.scale(modelViewMatrix,     // destination matrix
  //   modelViewMatrix,     // matrix to translate
  //   [0.02, 0.02, 1.0]);  // amount to translate

  mat4.translate(
    modelViewMatrix, // destination matrix
    modelViewMatrix, // matrix to translate
    [-3.0 - 20.0, -1.0 + 1.0, -16.0 - 50.0]
  ); // amount to translate

  mat4.rotateX(modelViewMatrix, modelViewMatrix, rotX);
  // mat4.rotateX(modelViewMatrix, modelViewMatrix, y);
  // mat4.rotateZ(modelViewMatrix, modelViewMatrix, x);
  rotX = 3.14 / 2;

  mat4.scale(
    modelViewMatrix, // destination matrix
    modelViewMatrix, // matrix to translate
    [0.1, 0.1, -3.0]
  );

  const viewMatrix = createViewMatrix();

  {
    const numComponents = 4;
    const type = gl.FLOAT;
    const normalize = false;
    const stride = 0;
    const offset = 0;
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.position);
    gl.vertexAttribPointer(
      programInfo.attribLocations.vertexPosition,
      numComponents,
      type,
      normalize,
      stride,
      offset
    );
    gl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition);

    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.normal);
    gl.vertexAttribPointer(
      programInfo.attribLocations.vertexNormal,
      3,
      type,
      true,
      stride,
      offset
    );
    gl.enableVertexAttribArray(programInfo.attribLocations.vertexNormal);
  }

  gl.useProgram(programInfo.program);

  gl.uniformMatrix4fv(
    programInfo.uniformLocations.projectionMatrix,
    false,
    projectionMatrix
  );
  gl.uniformMatrix4fv(
    programInfo.uniformLocations.modelViewMatrix,
    false,
    modelViewMatrix
  );
  gl.uniformMatrix4fv(
    programInfo.uniformLocations.viewMatrix,
    false,
    viewMatrix
  );

  gl.uniform3fv(programInfo.uniformLocations.colorVec, [0.8, 0.7, 0.6]);
  const lightSource = vec3.create();
  vec3.normalize(lightSource, vec3.fromValues(0, -0.5, 0.7));
  gl.uniform3fv(programInfo.uniformLocations.lightSourceVec, lightSource);
  {
    const offset = 0;
    const vertexCount = buffers.vertexCount;
    gl.drawArrays(gl.TRIANGLES, offset, vertexCount);
  }

  {
    const numComponents = 4;
    const type = gl.FLOAT;
    const normalize = false;
    const stride = 0;
    const offset = 0;
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.buildings);
    gl.vertexAttribPointer(
      programInfo.attribLocations.vertexPosition,
      numComponents,
      type,
      normalize,
      stride,
      offset
    );
    gl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition);

    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.buildingNormals);
    gl.vertexAttribPointer(
      programInfo.attribLocations.vertexNormal,
      3,
      type,
      true,
      stride,
      offset
    );
    gl.enableVertexAttribArray(programInfo.attribLocations.vertexNormal);
  }

  modelViewMatrix = mat4.create();
  mat4.translate(
    modelViewMatrix, // destination matrix
    modelViewMatrix, // matrix to translate
    [-3.0 - 20.0, -1.0 + 1.0 + 10, -16.0 - 50.0]
  ); // amount to translate

  mat4.rotateX(modelViewMatrix, modelViewMatrix, rotX);
  // mat4.rotateX(modelViewMatrix, modelViewMatrix, y);
  // mat4.rotateZ(modelViewMatrix, modelViewMatrix, x);
  rotX = 3.14 / 2;

  mat4.scale(
    modelViewMatrix, // destination matrix
    modelViewMatrix, // matrix to translate
    [0.1, 0.1, 3.0]
  );
  gl.uniformMatrix4fv(
    programInfo.uniformLocations.modelViewMatrix,
    false,
    modelViewMatrix
  );

  vec3.normalize(lightSource, vec3.fromValues(0, 0.3, -1));
  gl.uniform3fv(programInfo.uniformLocations.lightSourceVec, lightSource);
  // console.log('Drawing buildings: ', buffers.buildingCount);
  const colorRotationMatrix = mat4.create();
  for (let i = 0; i < buffers.buildingCount; ++i) {
    mat4.rotate(colorRotationMatrix, colorRotationMatrix, 0.1, [0.0, 1.0, 0.0]);
    const color = vec4.fromValues(0.8, 0.3, 0.8, 1);
    const normalized = vec4.create();
    const scaled = vec4.create();
    vec4.transformMat4(color, color, colorRotationMatrix);
    vec4.normalize(color, color);
    vec4.scale(color, color, 0.2);
    const baseColor = vec4.fromValues(0.9, 0.6, 0.55, 1);
    vec4.add(color, baseColor, color);
    gl.uniform3fv(programInfo.uniformLocations.colorVec, color.slice(0, 3));

    const offset = i * 36;
    gl.drawArrays(gl.TRIANGLES, offset, 36);
  }
}

@Component({
  selector: 'app-world',
  templateUrl: './world.component.html',
  styleUrls: ['./world.component.scss'],
})
export class WorldComponent implements OnInit {
  @ViewChild('mainCanvas') mainCanvas?: ElementRef<HTMLCanvasElement>;

  constructor(private http: HttpClient) {}

  // eslint-disable-next-line @typescript-eslint/no-empty-function
  ngOnInit(): void {}

  async ngAfterViewInit() {
    const canvas = this.mainCanvas?.nativeElement;
    if (!canvas) return;

    canvas.width = 1024;
    canvas.height = 1024;

    const gl = canvas.getContext('webgl');
    if (gl === null) {
      alert(
        'Unable to initialize WebGL. Your browser or machine may not support it.'
      );
      return;
    }

    const shaderProgram = initShaderProgram(gl, vsSource, fsSource);
    if (!shaderProgram) return;
    const programInfo = {
      program: shaderProgram,
      attribLocations: {
        vertexPosition: gl.getAttribLocation(shaderProgram, 'aVertexPosition'),
        vertexNormal: gl.getAttribLocation(shaderProgram, 'aVertexNormal'),
      },
      uniformLocations: {
        projectionMatrix: gl.getUniformLocation(
          shaderProgram,
          'uProjectionMatrix'
        ),
        modelViewMatrix: gl.getUniformLocation(
          shaderProgram,
          'uModelViewMatrix'
        ),
        viewMatrix: gl.getUniformLocation(shaderProgram, 'uViewMatrix'),
        lightSourceVec: gl.getUniformLocation(shaderProgram, 'uLightSource'),
        colorVec: gl.getUniformLocation(shaderProgram, 'uColor'),
      },
    };
    console.log(programInfo);

    const positions = await this.http
      .get<{ vertices: [number, number, number, number][] }>(
        'http://localhost:4200/assets/generated/height_model.json'
      )
      .pipe(first())
      .toPromise();
    const vertices = (positions?.vertices || [])
      .map((x) => [x[0], x[1], x[2], x[3]])
      .flatMap((x) => x);
    const normalList = await this.http
      .get<{ normals: [number, number, number][] }>(
        'http://localhost:4200/assets/generated/height_normals.json'
      )
      .pipe(first())
      .toPromise();
    const normals = (normalList?.normals || []).flatMap((x) => x);

    const buildings = (
      (
        await this.http
          .get<{ vertices: [number, number, number, number][][] }>(
            'http://localhost:4200/assets/generated/building_models.json'
          )
          .pipe(first())
          .toPromise()
      )?.vertices || []
    ).map((b) => b.flatMap((n) => n));

    const buildingNormals = (
      (
        await this.http
          .get<{ vertices: [number, number, number, number][][] }>(
            'http://localhost:4200/assets/generated/building_normals.json'
          )
          .pipe(first())
          .toPromise()
      )?.vertices || []
    ).map((b) => b.flatMap((n) => n));

    const buffers = initBuffers(
      gl,
      vertices,
      normals,
      buildings,
      buildingNormals
    );
    gl.clearColor(1.0, 0.0, 0.0, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    interval(100).subscribe(() => {
      drawScene(gl, programInfo, buffers);
      0;
    });
  }

  onMouseMove(event: MouseEvent) {
    if (event.buttons & 1) {
      if (event.shiftKey) {
        this.addMovement(
          vec3.fromValues(-event.movementX / 50, event.movementY / 50, 0)
        );
      } else {
        this.addRotation(-event.movementX / 500, -event.movementY / 500);
      }
    }
  }

  onMouseWheel(event: WheelEvent) {
    this.addMovement(vec3.fromValues(0, 0, event.deltaY / 200));
  }

  addRotation(dx: number, dy: number) {
    x += dx;
    y += dy;
  }

  addMovement(add: vec3) {
    const rotated = vec3.create();
    vec3.rotateX(rotated, add, vec3.fromValues(0, 0, 0), y);
    vec3.rotateY(rotated, rotated, vec3.fromValues(0, 0, 0), x);
    posX += rotated[0];
    posY += rotated[1];
    posZ += rotated[2];
  }
}

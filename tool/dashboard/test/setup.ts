import "@testing-library/jest-dom";

// Mock ResizeObserver for @xyflow/react
class ResizeObserverMock {
  observe() {}
  unobserve() {}
  disconnect() {}
}
window.ResizeObserver = window.ResizeObserver || ResizeObserverMock;

// Mock DOMMatrixReadOnly for @xyflow/react
class DOMMatrixReadOnlyMock {
  a = 1;
  b = 0;
  c = 0;
  d = 1;
  e = 0;
  f = 0;
  m11 = 1;
  m12 = 0;
  m13 = 0;
  m14 = 0;
  m21 = 0;
  m22 = 1;
  m23 = 0;
  m24 = 0;
  m31 = 0;
  m32 = 0;
  m33 = 1;
  m34 = 0;
  m41 = 0;
  m42 = 0;
  m43 = 0;
  m44 = 1;
  is2D = true;
  isIdentity = true;
  flipX() { return this; }
  flipY() { return this; }
  inverse() { return this; }
  multiply() { return this; }
  rotate() { return this; }
  rotateAxisAngle() { return this; }
  rotateFromVector() { return this; }
  scale() { return this; }
  scale3d() { return this; }
  scaleNonUniform() { return this; }
  skewX() { return this; }
  skewY() { return this; }
  toFloat32Array() { return new Float32Array(); }
  toFloat64Array() { return new Float64Array(); }
  toJSON() { return {}; }
  transformPoint() { return { x: 0, y: 0, z: 0, w: 1 }; }
  translate() { return this; }
}
(window as any).DOMMatrixReadOnly = window.DOMMatrixReadOnly || DOMMatrixReadOnlyMock;

// Mock matchMedia
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }),
});

// Mock IntersectionObserver
class IntersectionObserverMock {
  root = null;
  rootMargin = "";
  thresholds = [];
  observe() {}
  unobserve() {}
  disconnect() {}
  takeRecords() { return []; }
}
window.IntersectionObserver = window.IntersectionObserver || (IntersectionObserverMock as any);

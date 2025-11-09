// Jest setup file for testing environment
require('@testing-library/jest-dom');

// Mock Tauri API since it's not available in test environment
global.window = global.window || {};
global.window.__TAURI__ = {
  invoke: jest.fn(),
  event: {
    listen: jest.fn(),
    emit: jest.fn(),
  },
};

// Mock matchMedia (required by some UI components)
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: jest.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(), // deprecated
    removeListener: jest.fn(), // deprecated
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
});

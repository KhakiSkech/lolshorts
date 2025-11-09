/** @type {import('jest').Config} */
export default {
  preset: 'ts-jest',
  testEnvironment: 'jsdom',

  // Test file patterns
  testMatch: [
    '<rootDir>/src/**/*.test.{ts,tsx}', // Only run unit tests in src/
  ],

  // Ignore Playwright E2E tests
  testPathIgnorePatterns: [
    '/node_modules/',
    '/tests/e2e/', // Playwright E2E tests should run with `playwright test`
    '\\.spec\\.ts$', // Ignore all .spec.ts files (Playwright convention)
  ],

  // TypeScript and module transformation
  transform: {
    '^.+\\.tsx?$': ['ts-jest', {
      tsconfig: {
        jsx: 'react-jsx',
        esModuleInterop: true,
        allowSyntheticDefaultImports: true,
      },
    }],
  },

  // Module name mappers for CSS and assets
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1', // Path alias support
    '\\.(css|less|scss|sass)$': 'identity-obj-proxy', // Mock CSS imports
    '\\.(jpg|jpeg|png|gif|svg|woff|woff2|ttf|eot)$': '<rootDir>/__mocks__/fileMock.js', // Mock assets
  },

  // Resolve modules with extensions
  moduleDirectories: ['node_modules', 'src'],

  // Setup files
  setupFilesAfterEnv: ['<rootDir>/jest.setup.js'],

  // Coverage configuration
  collectCoverageFrom: [
    'src/**/*.{ts,tsx}',
    '!src/**/*.d.ts',
    '!src/**/*.stories.{ts,tsx}',
    '!src/**/*.test.{ts,tsx}',
    '!src/main.tsx', // Entry point
  ],

  // Module file extensions
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json'],
};

#!/usr/bin/env node

/**
 * ASCII Art Generation Script
 *
 * Generates ASCII art for the VibeTea logo using figlet with the "slant" font.
 * Outputs the result as a TypeScript constant for use in the application.
 */

import figlet from 'figlet';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const OUTPUT_DIR = path.join(__dirname, '..', 'src', 'assets', 'ascii');
const OUTPUT_FILE = path.join(OUTPUT_DIR, 'vibetea-logo.ts');

/**
 * Ensures the output directory exists, creating it recursively if necessary.
 */
function ensureDirectoryExists(dirPath) {
  if (!fs.existsSync(dirPath)) {
    fs.mkdirSync(dirPath, { recursive: true });
    console.log(`Created directory: ${dirPath}`);
  }
}

/**
 * Generates ASCII art text using figlet.
 */
function generateAsciiArt(text, font) {
  return figlet.textSync(text, { font });
}

/**
 * Escapes special characters for use in a template literal.
 * - Backslashes must be doubled
 * - Backticks must be escaped
 * - Dollar signs followed by { must be escaped to prevent interpolation
 */
function escapeForTemplateLiteral(str) {
  return str
    .replace(/\\/g, '\\\\')
    .replace(/`/g, '\\`')
    .replace(/\$\{/g, '\\${');
}

/**
 * Creates the TypeScript file content with the ASCII art as a constant.
 */
function createTypeScriptContent(asciiArt) {
  const escapedArt = escapeForTemplateLiteral(asciiArt);
  return `export const VIBETEA_ASCII = \`${escapedArt}\` as const;
`;
}

/**
 * Main execution function.
 */
function main() {
  try {
    // Ensure the output directory exists
    ensureDirectoryExists(OUTPUT_DIR);

    // Generate ASCII art with Slant font for developer aesthetic
    const asciiArt = generateAsciiArt('VibeTea', 'Slant');

    // Create TypeScript file content
    const content = createTypeScriptContent(asciiArt);

    // Write the output file
    fs.writeFileSync(OUTPUT_FILE, content, 'utf8');

    console.log(`ASCII art generated successfully!`);
    console.log(`Output: ${OUTPUT_FILE}`);
    console.log('\nGenerated art preview:');
    console.log(asciiArt);
  } catch (error) {
    console.error('Error generating ASCII art:', error.message);
    process.exit(1);
  }
}

main();

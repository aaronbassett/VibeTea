#!/usr/bin/env node
/* global console, process */

/**
 * Reduced Motion Compliance Checker
 *
 * Scans component files for animation usage (framer-motion, CSS animations)
 * and verifies that useReducedMotion hook is imported where animations are used.
 *
 * Exit codes:
 *   0 - All files with animations have reduced motion support
 *   1 - Some files are missing reduced motion support
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/** Root directory for component scanning */
const SRC_DIR = path.join(__dirname, '..', 'src');

/** Patterns that indicate animation usage requiring reduced motion support */
const ANIMATION_PATTERNS = [
  // framer-motion imports
  /from\s+['"]framer-motion['"]/,
  // framer-motion component usage (m., motion., AnimatePresence, etc.)
  /<(m\.|motion\.|AnimatePresence|LayoutGroup)/,
  // CSS animation properties in style objects
  /animation:\s*['"`]/,
  /animationName:\s*['"`]/,
  /transition:\s*['"`](?!none)/,
  // CSS-in-JS animation keyframes
  /@keyframes/,
  // Inline style with animation
  /style=\{[^}]*animation/,
];

/** Pattern that indicates reduced motion support is present */
const REDUCED_MOTION_PATTERN = /useReducedMotion/;

/** Files to exclude from scanning */
const EXCLUDE_PATTERNS = [
  // Test files
  /\.test\.(ts|tsx)$/,
  /\.spec\.(ts|tsx)$/,
  /__tests__/,
  // Story files (they may demonstrate animations intentionally)
  /\.stories\.(ts|tsx)$/,
  // Type definition files
  /\.d\.ts$/,
  // Hook files (utility hooks don't render UI directly)
  /hooks\//,
  // Non-component files
  /types\//,
  /constants\//,
  /utils\//,
  /assets\//,
];

/**
 * Recursively finds all TypeScript/TSX files in a directory.
 *
 * @param {string} dir - Directory to search
 * @param {string[]} files - Accumulator for found files
 * @returns {string[]} Array of file paths
 */
function findFiles(dir, files = []) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      findFiles(fullPath, files);
    } else if (entry.isFile() && /\.(ts|tsx)$/.test(entry.name)) {
      files.push(fullPath);
    }
  }

  return files;
}

/**
 * Checks if a file path should be excluded from scanning.
 *
 * @param {string} filePath - Path to check
 * @returns {boolean} True if the file should be excluded
 */
function shouldExclude(filePath) {
  const relativePath = path.relative(SRC_DIR, filePath);
  return EXCLUDE_PATTERNS.some((pattern) => pattern.test(relativePath));
}

/**
 * Checks if a file contains animation patterns.
 *
 * @param {string} content - File content to check
 * @returns {string[]} Array of matched animation patterns
 */
function findAnimationUsage(content) {
  const matches = [];

  for (const pattern of ANIMATION_PATTERNS) {
    if (pattern.test(content)) {
      // Extract the pattern name for reporting
      const match = content.match(pattern);
      if (match) {
        matches.push(match[0].trim().slice(0, 50));
      }
    }
  }

  return matches;
}

/**
 * Checks if a file has reduced motion support.
 *
 * @param {string} content - File content to check
 * @returns {boolean} True if reduced motion support is present
 */
function hasReducedMotionSupport(content) {
  return REDUCED_MOTION_PATTERN.test(content);
}

/**
 * Main execution function.
 */
function main() {
  console.log('Reduced Motion Compliance Check');
  console.log('================================\n');
  console.log(`Scanning: ${SRC_DIR}\n`);

  // Find all TypeScript/TSX files
  const allFiles = findFiles(SRC_DIR);
  const filesToCheck = allFiles.filter((file) => !shouldExclude(file));

  console.log(`Found ${allFiles.length} total files`);
  console.log(`Checking ${filesToCheck.length} component files (excluding tests, stories, types)\n`);

  /** @type {{ file: string; animations: string[] }[]} */
  const violations = [];

  /** @type {{ file: string; animations: string[] }[]} */
  const compliant = [];

  for (const filePath of filesToCheck) {
    const content = fs.readFileSync(filePath, 'utf-8');
    const animationUsage = findAnimationUsage(content);

    if (animationUsage.length > 0) {
      const relativePath = path.relative(SRC_DIR, filePath);

      if (hasReducedMotionSupport(content)) {
        compliant.push({ file: relativePath, animations: animationUsage });
      } else {
        violations.push({ file: relativePath, animations: animationUsage });
      }
    }
  }

  // Report compliant files
  if (compliant.length > 0) {
    console.log('Compliant files (have animation + reduced motion support):');
    for (const { file, animations } of compliant) {
      console.log(`  [OK] ${file}`);
      console.log(`       Animations: ${animations.join(', ')}`);
    }
    console.log();
  }

  // Report violations
  if (violations.length > 0) {
    console.log('VIOLATIONS (animation usage without useReducedMotion):');
    for (const { file, animations } of violations) {
      console.log(`  [FAIL] ${file}`);
      console.log(`         Animations: ${animations.join(', ')}`);
    }
    console.log();
    console.log(`Found ${violations.length} file(s) with animations missing reduced motion support.`);
    console.log('\nTo fix: Import and use the useReducedMotion hook from ../hooks/useReducedMotion');
    console.log('Example:');
    console.log('  const prefersReducedMotion = useReducedMotion();');
    console.log('  // Then conditionally disable/simplify animations based on this value');
    process.exit(1);
  }

  console.log('All files with animations have reduced motion support.');
  console.log(`Checked ${filesToCheck.length} files, ${compliant.length} use animations.`);
  process.exit(0);
}

main();

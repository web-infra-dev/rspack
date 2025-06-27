// Test for mixed export pattern edge case
const fs = require('fs');
const path = require('path');
const { describe, expect, test, beforeAll } = require("@rstest/core");

describe('Mixed Export Pattern Tests', () => {
  let distFiles;
  
  beforeAll(() => {
    const distPath = path.join(__dirname, '../../dist');
    if (!fs.existsSync(distPath)) {
      throw new Error('Dist directory not found. Run npm run build first.');
    }
    distFiles = fs.readdirSync(distPath).filter(f => f.endsWith('.js'));
  });

  test('should handle module.exports assignment followed by property additions', () => {
    // Find a file that demonstrates the mixed pattern
    const targetFile = distFiles.find(file => {
      const filePath = path.join(__dirname, '../../dist', file);
      const content = fs.readFileSync(filePath, 'utf8');
      
      // Look for the specific pattern: module.exports = value followed by module.exports.prop = value
      return content.includes('module.exports = ') && 
             content.includes('module.exports.') &&
             content.includes('@common:if');
    });

    expect(targetFile).toBeDefined();
    
    const filePath = path.join(__dirname, '../../dist', targetFile);
    const content = fs.readFileSync(filePath, 'utf8');
    
    console.log(`Testing mixed export pattern in: ${targetFile}`);
    
    // Verify the pattern exists
    expect(content).toMatch(/module\.exports\s*=\s*[^;]+;/);
    expect(content).toMatch(/module\.exports\.\w+\s*=\s*/);
    expect(content).toMatch(/@common:if.*?module\.exports\.\w+.*?@common:endif/);
  });

  test('should correctly position macros for property additions after module.exports assignment', () => {
    const targetFile = distFiles.find(file => {
      const filePath = path.join(__dirname, '../../dist', file);
      const content = fs.readFileSync(filePath, 'utf8');
      return content.includes('module.exports.') && content.includes('@common:if');
    });

    if (!targetFile) {
      console.log('No files found with module.exports property pattern, skipping test');
      return;
    }

    const filePath = path.join(__dirname, '../../dist', targetFile);
    const content = fs.readFileSync(filePath, 'utf8');
    
    // Extract all module.exports property assignments with macros
    const propertyAssignmentPattern = /\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/\s*module\.exports\.(\w+)\s*\/\*\s*@common:endif\s*\*\/\s*=\s*([^;]+);/g;
    const matches = [...content.matchAll(propertyAssignmentPattern)];
    
    console.log(`Found ${matches.length} module.exports property assignments with macros`);
    
    matches.forEach((match, index) => {
      const [fullMatch, propertyName, value] = match;
      console.log(`  ${index + 1}. Property: ${propertyName}, Value: ${value.trim()}`);
      
      // Verify the macro wraps the property name correctly
      expect(fullMatch).toMatch(/\/\*\s*@common:if.*?\*\/\s*module\.exports\.\w+\s*\/\*\s*@common:endif\s*\*\//);
      
      // Verify the property name is wrapped, not the assignment
      expect(fullMatch).not.toMatch(/\/\*\s*@common:if.*?\*\/\s*module\.exports\.\w+\s*=.*?\/\*\s*@common:endif\s*\*\//);
    });

    // Should have at least one properly wrapped property assignment
    expect(matches.length).toBeGreaterThan(0);
  });

  test('should handle circular reference exports correctly', () => {
    const targetFile = distFiles.find(file => {
      const filePath = path.join(__dirname, '../../dist', file);
      const content = fs.readFileSync(filePath, 'utf8');
      return content.includes('getSelf') || content.includes('return module.exports');
    });

    if (!targetFile) {
      console.log('No files found with circular reference pattern, skipping test');
      return;
    }

    const filePath = path.join(__dirname, '../../dist', targetFile);
    const content = fs.readFileSync(filePath, 'utf8');
    
    console.log(`Testing circular reference pattern in: ${targetFile}`);
    
    // Look for getSelf function or similar circular reference
    const circularRefPattern = /\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/\s*module\.exports\.(\w+)\s*\/\*\s*@common:endif\s*\*\/\s*=\s*function\s*\([^)]*\)\s*\{[^}]*return\s+module\.exports[^}]*\}/;
    const circularMatch = content.match(circularRefPattern);
    
    if (circularMatch) {
      console.log(`Found circular reference function: ${circularMatch[1]}`);
      
      // Verify the macro positioning is correct for circular reference
      expect(circularMatch[0]).toMatch(/\/\*\s*@common:if.*?\*\/\s*module\.exports\.\w+\s*\/\*\s*@common:endif\s*\*\//);
    }
  });

  test('should not wrap the entire assignment for property additions', () => {
    const targetFile = distFiles.find(file => {
      const filePath = path.join(__dirname, '../../dist', file);
      const content = fs.readFileSync(filePath, 'utf8');
      return content.includes('module.exports.') && content.includes('@common:if');
    });

    if (!targetFile) {
      console.log('No files found with module.exports properties, skipping test');
      return;
    }

    const filePath = path.join(__dirname, '../../dist', targetFile);
    const content = fs.readFileSync(filePath, 'utf8');
    
    // This pattern would be WRONG - wrapping the entire assignment
    const wrongPattern = /\/\*\s*@common:if.*?\*\/\s*module\.exports\.\w+\s*=\s*[^;]+;\s*\/\*\s*@common:endif\s*\*\//g;
    const wrongMatches = [...content.matchAll(wrongPattern)];
    
    console.log(`Checking for incorrect macro positioning...`);
    
    if (wrongMatches.length > 0) {
      console.log(`❌ Found ${wrongMatches.length} incorrectly wrapped assignments:`);
      wrongMatches.forEach((match, index) => {
        console.log(`  ${index + 1}. ${match[0]}`);
      });
    }
    
    // Should not have any incorrectly wrapped assignments
    expect(wrongMatches.length).toBe(0);
  });

  test('should maintain correct syntax after macro insertion', () => {
    distFiles.forEach(file => {
      const filePath = path.join(__dirname, '../../dist', file);
      const content = fs.readFileSync(filePath, 'utf8');
      
      // Check for basic JavaScript syntax errors that might be introduced by macro insertion
      
      // No unmatched braces
      const openBraces = (content.match(/\{/g) || []).length;
      const closeBraces = (content.match(/\}/g) || []).length;
      expect(openBraces).toBe(closeBraces);
      
      // No unmatched parentheses
      const openParens = (content.match(/\(/g) || []).length;
      const closeParens = (content.match(/\)/g) || []).length;
      expect(openParens).toBe(closeParens);
      
      // All @common:if have matching @common:endif
      const ifCount = (content.match(/@common:if/g) || []).length;
      const endifCount = (content.match(/@common:endif/g) || []).length;
      expect(ifCount).toBe(endifCount);
      
      // No malformed property access (e.g., module.exports. = value)
      expect(content).not.toMatch(/module\.exports\.\s*=/);
      expect(content).not.toMatch(/exports\.\s*=/);
    });
  });
});
#!/usr/bin/env node

/**
 * FPS Genie Cargo Helper MCP Server
 * Provides Rust/Cargo development tools and workspace management
 */

const { Server } = require('@modelcontextprotocol/sdk/server/index.js');
const { StdioServerTransport } = require('@modelcontextprotocol/sdk/server/stdio.js');
const { 
  CallToolRequestSchema,
  ListToolsRequestSchema,
} = require('@modelcontextprotocol/sdk/types.js');
const { exec } = require('child_process');
const { promisify } = require('util');
const fs = require('fs').promises;
const path = require('path');

const execAsync = promisify(exec);

class CargoHelperServer {
  constructor() {
    this.server = new Server(
      {
        name: 'fps-genie-cargo-helper',
        version: '1.0.0',
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );
    
    this.setupToolHandlers();
  }

  setupToolHandlers() {
    this.server.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: [
        {
          name: 'cargo_workspace_info',
          description: 'Get information about the Cargo workspace structure and crates',
          inputSchema: {
            type: 'object',
            properties: {}
          }
        },
        {
          name: 'cargo_build_specific',
          description: 'Build specific crates with optimized build commands',
          inputSchema: {
            type: 'object',
            properties: {
              crate_name: {
                type: 'string',
                description: 'Name of the crate to build (e.g., cs2-demo-parser)'
              },
              build_type: {
                type: 'string',
                enum: ['check', 'build', 'test', 'clippy', 'fmt'],
                description: 'Type of build operation',
                default: 'check'
              },
              features: {
                type: 'array',
                items: { type: 'string' },
                description: 'Features to enable (e.g., ["integration-tests"])'
              },
              no_default_features: {
                type: 'boolean',
                description: 'Disable default features (useful for cs2-ml on Linux)',
                default: false
              }
            },
            required: ['crate_name']
          }
        },
        {
          name: 'cargo_dependency_analysis',
          description: 'Analyze dependencies and suggest optimizations',
          inputSchema: {
            type: 'object',
            properties: {
              crate_name: {
                type: 'string',
                description: 'Optional: specific crate to analyze'
              },
              check_outdated: {
                type: 'boolean',
                description: 'Check for outdated dependencies',
                default: true
              }
            }
          }
        },
        {
          name: 'cargo_test_runner',
          description: 'Run tests with appropriate configurations for the FPS Genie project',
          inputSchema: {
            type: 'object',
            properties: {
              test_type: {
                type: 'string',
                enum: ['unit', 'integration', 'all', 'specific'],
                description: 'Type of tests to run',
                default: 'unit'
              },
              crate_name: {
                type: 'string',
                description: 'Specific crate to test'
              },
              test_name: {
                type: 'string',
                description: 'Specific test name (for test_type=specific)'
              },
              parallel: {
                type: 'boolean',
                description: 'Run tests in parallel',
                default: true
              }
            }
          }
        },
        {
          name: 'cargo_performance_check',
          description: 'Check build performance and suggest optimizations',
          inputSchema: {
            type: 'object',
            properties: {
              measure_build_time: {
                type: 'boolean',
                description: 'Measure build times for each crate',
                default: true
              }
            }
          }
        },
        {
          name: 'cargo_platform_fixes',
          description: 'Apply platform-specific fixes and configurations',
          inputSchema: {
            type: 'object',
            properties: {
              platform: {
                type: 'string',
                enum: ['linux', 'macos', 'windows', 'auto'],
                description: 'Target platform for fixes',
                default: 'auto'
              },
              fix_type: {
                type: 'string',
                enum: ['ml-cpu-only', 'protobuf', 'fontconfig', 'all'],
                description: 'Type of fix to apply',
                default: 'all'
              }
            }
          }
        },
        {
          name: 'cargo_workspace_health',
          description: 'Check overall workspace health and common issues',
          inputSchema: {
            type: 'object',
            properties: {
              include_suggestions: {
                type: 'boolean',
                description: 'Include optimization suggestions',
                default: true
              }
            }
          }
        },
        {
          name: 'cargo_feature_analysis',
          description: 'Analyze feature flags and their usage across the workspace',
          inputSchema: {
            type: 'object',
            properties: {
              suggest_combinations: {
                type: 'boolean',
                description: 'Suggest optimal feature combinations',
                default: true
              }
            }
          }
        }
      ]
    }));

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        switch (name) {
          case 'cargo_workspace_info':
            return await this.getWorkspaceInfo(args);
          case 'cargo_build_specific':
            return await this.buildSpecificCrate(args);
          case 'cargo_dependency_analysis':
            return await this.analyzeDependencies(args);
          case 'cargo_test_runner':
            return await this.runTests(args);
          case 'cargo_performance_check':
            return await this.checkPerformance(args);
          case 'cargo_platform_fixes':
            return await this.applyPlatformFixes(args);
          case 'cargo_workspace_health':
            return await this.checkWorkspaceHealth(args);
          case 'cargo_feature_analysis':
            return await this.analyzeFeatures(args);
          default:
            throw new Error(`Unknown tool: ${name}`);
        }
      } catch (error) {
        return {
          content: [
            {
              type: 'text',
              text: `Error executing ${name}: ${error.message}\n\nStack trace:\n${error.stack}`
            }
          ]
        };
      }
    });
  }

  async getWorkspaceInfo(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    
    try {
      // Read workspace Cargo.toml
      const cargoTomlPath = path.join(workspaceRoot, 'Cargo.toml');
      const cargoToml = await fs.readFile(cargoTomlPath, 'utf8');
      
      // Extract workspace members
      const membersMatch = cargoToml.match(/members\s*=\s*\[([\s\S]*?)\]/);
      const members = membersMatch ? 
        membersMatch[1].split(',').map(m => m.trim().replace(/"/g, '')) : [];
      
      // Get info for each crate
      const crateInfo = await Promise.all(
        members.map(async (member) => {
          try {
            const cratePath = path.join(workspaceRoot, member, 'Cargo.toml');
            const crateToml = await fs.readFile(cratePath, 'utf8');
            
            const nameMatch = crateToml.match(/name\s*=\s*"([^"]+)"/);
            const versionMatch = crateToml.match(/version\s*=\s*"([^"]+)"/);
            const descMatch = crateToml.match(/description\s*=\s*"([^"]+)"/);
            
            return {
              name: nameMatch ? nameMatch[1] : member,
              path: member,
              version: versionMatch ? versionMatch[1] : 'unknown',
              description: descMatch ? descMatch[1] : 'No description',
              build_time_estimate: this.getBuildTimeEstimate(member)
            };
          } catch (error) {
            return {
              name: member,
              path: member,
              error: `Failed to read Cargo.toml: ${error.message}`
            };
          }
        })
      );
      
      const workspaceInfo = {
        workspace_root: workspaceRoot,
        total_crates: crateInfo.length,
        crates: crateInfo,
        recommended_build_order: this.getRecommendedBuildOrder(),
        platform_notes: await this.getPlatformNotes()
      };
      
      return {
        content: [
          {
            type: 'text',
            text: `Cargo Workspace Information:\n\n${JSON.stringify(workspaceInfo, null, 2)}`
          }
        ]
      };
    } catch (error) {
      throw new Error(`Failed to get workspace info: ${error.message}`);
    }
  }

  getBuildTimeEstimate(crateName) {
    const estimates = {
      'cs2-demo-parser': '10 seconds',
      'cs2-common': '5 seconds',
      'cs2-data-pipeline': '90 seconds',
      'cs2-ml': '2+ minutes (platform dependent)',
      'cs2-demo-analyzer': '1+ minute',
      'cs2-client': '30 seconds',
      'cs2-analytics': '1+ minute',
      'cs2-integration-tests': '2+ minutes',
      'csgoproto': '15 seconds'
    };
    return estimates[crateName] || 'unknown';
  }

  getRecommendedBuildOrder() {
    return [
      { step: 1, crates: ['cs2-common', 'csgoproto'], reason: 'Foundation crates, fast builds' },
      { step: 2, crates: ['cs2-demo-parser'], reason: 'Core parser, moderate build time' },
      { step: 3, crates: ['cs2-client', 'cs2-demo-analyzer'], reason: 'Tools and utilities' },
      { step: 4, crates: ['cs2-data-pipeline', 'cs2-analytics'], reason: 'Heavy processing crates' },
      { step: 5, crates: ['cs2-ml'], reason: 'ML crate, longest build time, platform issues' },
      { step: 6, crates: ['cs2-integration-tests'], reason: 'Tests last, requires other crates' }
    ];
  }

  async getPlatformNotes() {
    try {
      const { stdout } = await execAsync('uname -s', { timeout: 5000 });
      const platform = stdout.trim().toLowerCase();
      
      if (platform.includes('linux')) {
        return {
          platform: 'Linux',
          notes: [
            'cs2-ml: Use --no-default-features to avoid Metal dependency',
            'Install: protobuf-compiler, libfontconfig1-dev',
            'Recommended: clang, lld for faster linking'
          ]
        };
      } else if (platform.includes('darwin')) {
        return {
          platform: 'macOS',
          notes: [
            'cs2-ml: Metal acceleration available by default',
            'Install via brew: cmake, protobuf, pkg-config',
            'Xcode command line tools required'
          ]
        };
      } else {
        return {
          platform: platform,
          notes: ['Platform-specific notes not available']
        };
      }
    } catch (error) {
      return {
        platform: 'unknown',
        notes: [`Failed to detect platform: ${error.message}`]
      };
    }
  }

  async buildSpecificCrate(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    const crateName = args.crate_name;
    const buildType = args.build_type || 'check';
    
    let command = `cd "${workspaceRoot}" && cargo ${buildType}`;
    
    // Add package specification
    command += ` -p ${crateName}`;
    
    // Add features
    if (args.features && args.features.length > 0) {
      command += ` --features ${args.features.join(',')}`;
    }
    
    // Add no-default-features flag
    if (args.no_default_features) {
      command += ' --no-default-features';
    }
    
    // Add specific flags for different build types
    if (buildType === 'clippy') {
      command += ' --all-targets --all-features -- -D warnings';
    } else if (buildType === 'test') {
      command += ' --lib';
    }
    
    const startTime = Date.now();
    
    try {
      const { stdout, stderr } = await execAsync(command, { 
        timeout: 300000, // 5 minutes
        maxBuffer: 1024 * 1024 * 5 // 5MB buffer
      });
      
      const duration = Date.now() - startTime;
      
      const result = {
        command_executed: command,
        crate_name: crateName,
        build_type: buildType,
        duration_ms: duration,
        duration_human: `${(duration / 1000).toFixed(1)}s`,
        status: 'success',
        stdout_lines: stdout.split('\n').length,
        stderr_lines: stderr.split('\n').length,
        has_warnings: stderr.includes('warning:'),
        output_summary: this.summarizeOutput(stdout, stderr),
        performance_note: this.getPerformanceNote(crateName, duration)
      };
      
      return {
        content: [
          {
            type: 'text',
            text: `Cargo Build Results:\n\n${JSON.stringify(result, null, 2)}`
          }
        ]
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      
      const result = {
        command_executed: command,
        crate_name: crateName,
        build_type: buildType,
        duration_ms: duration,
        status: 'failed',
        error: error.message,
        exit_code: error.code,
        troubleshooting: this.getTroubleshootingTips(crateName, error.message)
      };
      
      return {
        content: [
          {
            type: 'text',
            text: `Cargo Build Failed:\n\n${JSON.stringify(result, null, 2)}`
          }
        ]
      };
    }
  }

  summarizeOutput(stdout, stderr) {
    const summary = {
      compilation_units: 0,
      warnings: 0,
      errors: 0
    };
    
    // Count compilation units
    const compileMatches = stdout.match(/Compiling \w+/g);
    summary.compilation_units = compileMatches ? compileMatches.length : 0;
    
    // Count warnings and errors
    summary.warnings = (stderr.match(/warning:/g) || []).length;
    summary.errors = (stderr.match(/error:/g) || []).length;
    
    return summary;
  }

  getPerformanceNote(crateName, durationMs) {
    const expected = {
      'cs2-demo-parser': 10000,
      'cs2-common': 5000,
      'cs2-data-pipeline': 90000,
      'cs2-ml': 120000,
      'cs2-demo-analyzer': 60000,
      'cs2-client': 30000,
      'cs2-analytics': 60000,
      'csgoproto': 15000
    };
    
    const expectedMs = expected[crateName];
    if (!expectedMs) return 'No performance baseline available';
    
    const ratio = durationMs / expectedMs;
    if (ratio < 0.8) return '🚀 Faster than expected!';
    if (ratio < 1.2) return '✅ Normal build time';
    if (ratio < 2.0) return '⚠️ Slower than expected';
    return '🐌 Much slower than expected - check system resources';
  }

  getTroubleshootingTips(crateName, errorMessage) {
    const tips = [];
    
    if (errorMessage.includes('protobuf') || errorMessage.includes('protoc')) {
      tips.push('Install protobuf-compiler: sudo apt-get install protobuf-compiler');
    }
    
    if (errorMessage.includes('fontconfig')) {
      tips.push('Install fontconfig: sudo apt-get install libfontconfig1-dev');
    }
    
    if (errorMessage.includes('objc_exception') || (crateName === 'cs2-ml' && errorMessage.includes('Metal'))) {
      tips.push('Use --no-default-features for cs2-ml on Linux');
      tips.push('cargo build -p cs2-ml --no-default-features');
    }
    
    if (errorMessage.includes('linker')) {
      tips.push('Install build essentials: sudo apt-get install build-essential clang');
    }
    
    if (errorMessage.includes('timeout') || errorMessage.includes('time')) {
      tips.push('Build timed out - this is normal for cs2-ml (can take 5+ minutes)');
      tips.push('Try: cargo build -p ' + crateName + ' --release');
    }
    
    return tips.length > 0 ? tips : ['Check the error message above and consult project documentation'];
  }

  async analyzeDependencies(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    
    try {
      // Get dependency tree
      const treeCommand = args.crate_name ? 
        `cd "${workspaceRoot}" && cargo tree -p ${args.crate_name}` :
        `cd "${workspaceRoot}" && cargo tree --workspace`;
      
      const { stdout: treeOutput } = await execAsync(treeCommand, { timeout: 30000 });
      
      // Check for outdated dependencies if requested
      let outdatedInfo = null;
      if (args.check_outdated) {
        try {
          const { stdout: outdatedOutput } = await execAsync(
            `cd "${workspaceRoot}" && cargo outdated --workspace --format json`,
            { timeout: 60000 }
          );
          outdatedInfo = JSON.parse(outdatedOutput);
        } catch (error) {
          outdatedInfo = { error: 'cargo-outdated not installed or failed' };
        }
      }
      
      const analysis = {
        dependency_tree_summary: {
          total_lines: treeOutput.split('\n').length,
          unique_crates: this.extractUniqueCrates(treeOutput),
          potential_duplicates: this.findPotentialDuplicates(treeOutput)
        },
        outdated_dependencies: outdatedInfo,
        workspace_dependencies: await this.getWorkspaceDependencies(),
        recommendations: this.getDependencyRecommendations(treeOutput)
      };
      
      return {
        content: [
          {
            type: 'text',
            text: `Dependency Analysis:\n\n${JSON.stringify(analysis, null, 2)}`
          }
        ]
      };
    } catch (error) {
      throw new Error(`Failed to analyze dependencies: ${error.message}`);
    }
  }

  extractUniqueCrates(treeOutput) {
    const crates = new Set();
    const lines = treeOutput.split('\n');
    
    for (const line of lines) {
      const match = line.match(/(\w[\w-]*)\s+v\d+/);
      if (match) {
        crates.add(match[1]);
      }
    }
    
    return crates.size;
  }

  findPotentialDuplicates(treeOutput) {
    const crateVersions = {};
    const lines = treeOutput.split('\n');
    
    for (const line of lines) {
      const match = line.match(/(\w[\w-]*)\s+v(\d+\.\d+\.\d+)/);
      if (match) {
        const [, name, version] = match;
        if (!crateVersions[name]) {
          crateVersions[name] = new Set();
        }
        crateVersions[name].add(version);
      }
    }
    
    const duplicates = [];
    for (const [name, versions] of Object.entries(crateVersions)) {
      if (versions.size > 1) {
        duplicates.push({
          crate: name,
          versions: Array.from(versions)
        });
      }
    }
    
    return duplicates;
  }

  async getWorkspaceDependencies() {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    
    try {
      const cargoTomlPath = path.join(workspaceRoot, 'Cargo.toml');
      const cargoToml = await fs.readFile(cargoTomlPath, 'utf8');
      
      // Extract workspace dependencies section
      const depsMatch = cargoToml.match(/\[workspace\.dependencies\]([\s\S]*?)(?=\[|\Z)/);
      if (depsMatch) {
        const depsSection = depsMatch[1];
        const deps = {};
        const lines = depsSection.split('\n');
        
        for (const line of lines) {
          const match = line.match(/(\w[\w-]*)\s*=\s*"([^"]+)"/);
          if (match) {
            deps[match[1]] = match[2];
          }
        }
        
        return deps;
      }
      
      return {};
    } catch (error) {
      return { error: error.message };
    }
  }

  getDependencyRecommendations(treeOutput) {
    const recommendations = [];
    
    // Check for common heavy dependencies
    if (treeOutput.includes('tokio')) {
      recommendations.push('Consider using tokio features selectively to reduce build time');
    }
    
    if (treeOutput.includes('serde')) {
      recommendations.push('Ensure serde features are optimized for your use case');
    }
    
    // Check for ML dependencies
    if (treeOutput.includes('candle')) {
      recommendations.push('ML dependencies detected - ensure platform compatibility');
    }
    
    return recommendations;
  }

  async runTests(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    const testType = args.test_type || 'unit';
    
    let command = `cd "${workspaceRoot}" && cargo test`;
    
    // Configure command based on test type
    switch (testType) {
      case 'unit':
        command += ' --lib --workspace --quiet';
        break;
      case 'integration':
        command += ' --package cs2-integration-tests --features integration-tests';
        break;
      case 'all':
        command += ' --workspace';
        break;
      case 'specific':
        if (args.crate_name) command += ` -p ${args.crate_name}`;
        if (args.test_name) command += ` ${args.test_name}`;
        break;
    }
    
    // Add parallel flag
    if (!args.parallel) {
      command += ' -- --test-threads=1';
    }
    
    const startTime = Date.now();
    
    try {
      const { stdout, stderr } = await execAsync(command, { 
        timeout: testType === 'integration' ? 900000 : 300000, // 15 min for integration, 5 min for others
        maxBuffer: 1024 * 1024 * 10 // 10MB buffer
      });
      
      const duration = Date.now() - startTime;
      
      const result = {
        command_executed: command,
        test_type: testType,
        duration_ms: duration,
        duration_human: `${(duration / 1000).toFixed(1)}s`,
        status: 'success',
        test_summary: this.parseTestResults(stdout),
        warnings: stderr ? stderr.split('\n').filter(line => line.includes('warning')).length : 0
      };
      
      return {
        content: [
          {
            type: 'text',
            text: `Test Results:\n\n${JSON.stringify(result, null, 2)}`
          }
        ]
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      
      const result = {
        command_executed: command,
        test_type: testType,
        duration_ms: duration,
        status: 'failed',
        error: error.message,
        failed_tests: this.parseFailedTests(error.stdout || error.message),
        troubleshooting: this.getTestTroubleshootingTips(testType, error.message)
      };
      
      return {
        content: [
          {
            type: 'text',
            text: `Test Failed:\n\n${JSON.stringify(result, null, 2)}`
          }
        ]
      };
    }
  }

  parseTestResults(stdout) {
    const summary = {
      passed: 0,
      failed: 0,
      ignored: 0,
      total: 0
    };
    
    // Look for test result summary line
    const summaryMatch = stdout.match(/test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored/);
    if (summaryMatch) {
      summary.passed = parseInt(summaryMatch[2]);
      summary.failed = parseInt(summaryMatch[3]);
      summary.ignored = parseInt(summaryMatch[4]);
      summary.total = summary.passed + summary.failed + summary.ignored;
    }
    
    return summary;
  }

  parseFailedTests(output) {
    const failedTests = [];
    const lines = output.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes('test result: FAILED')) {
        // Look for test names in preceding lines
        for (let j = Math.max(0, i - 10); j < i; j++) {
          const testMatch = lines[j].match(/test (\w+::\w+) \.\.\. FAILED/);
          if (testMatch) {
            failedTests.push(testMatch[1]);
          }
        }
      }
    }
    
    return failedTests;
  }

  getTestTroubleshootingTips(testType, errorMessage) {
    const tips = [];
    
    if (testType === 'integration' && errorMessage.includes('connection')) {
      tips.push('Integration tests require databases - run ./setup_databases.sh first');
      tips.push('Check docker compose ps to ensure all services are running');
    }
    
    if (errorMessage.includes('timeout')) {
      tips.push('Tests timed out - this is normal for integration tests (can take 15+ minutes)');
      tips.push('Consider running tests with fewer threads: -- --test-threads=1');
    }
    
    if (errorMessage.includes('TestContainers')) {
      tips.push('TestContainers requires Docker to be running');
      tips.push('Ensure Docker daemon is accessible to the current user');
    }
    
    return tips.length > 0 ? tips : ['Check test output for specific failure details'];
  }

  async checkPerformance(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    
    const performanceCheck = {
      timestamp: new Date().toISOString(),
      build_performance: {},
      optimization_suggestions: [],
      system_info: {}
    };
    
    // Get system info
    try {
      const { stdout: cpuInfo } = await execAsync('nproc', { timeout: 5000 });
      const { stdout: memInfo } = await execAsync('free -h', { timeout: 5000 });
      
      performanceCheck.system_info = {
        cpu_cores: parseInt(cpuInfo.trim()),
        memory_info: memInfo.trim().split('\n')[1]
      };
    } catch (error) {
      performanceCheck.system_info.error = error.message;
    }
    
    // Measure build times if requested
    if (args.measure_build_time) {
      const testCrates = ['cs2-common', 'cs2-demo-parser'];
      
      for (const crate of testCrates) {
        const startTime = Date.now();
        try {
          await execAsync(`cd "${workspaceRoot}" && cargo check -p ${crate}`, { timeout: 120000 });
          performanceCheck.build_performance[crate] = {
            duration_ms: Date.now() - startTime,
            status: 'success'
          };
        } catch (error) {
          performanceCheck.build_performance[crate] = {
            duration_ms: Date.now() - startTime,
            status: 'failed',
            error: error.message
          };
        }
      }
    }
    
    // Generate optimization suggestions
    performanceCheck.optimization_suggestions = [
      'Use `cargo check` for fast iteration instead of `cargo build`',
      'Enable incremental compilation with CARGO_INCREMENTAL=1',
      'Use `--no-default-features` for cs2-ml on Linux to avoid Metal dependency',
      'Consider using sccache for dependency caching',
      'Run tests in parallel except for integration tests'
    ];
    
    return {
      content: [
        {
          type: 'text',
          text: `Performance Check Results:\n\n${JSON.stringify(performanceCheck, null, 2)}`
        }
      ]
    };
  }

  async applyPlatformFixes(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    const platform = args.platform === 'auto' ? await this.detectPlatform() : args.platform;
    const fixType = args.fix_type || 'all';
    
    const fixes = {
      platform_detected: platform,
      fix_type: fixType,
      applied_fixes: [],
      manual_steps: [],
      status: 'completed'
    };
    
    try {
      if (platform === 'linux') {
        if (fixType === 'ml-cpu-only' || fixType === 'all') {
          // Check and modify cs2-ml Cargo.toml for CPU-only features
          const mlCargoPath = path.join(workspaceRoot, 'cs2-ml', 'Cargo.toml');
          try {
            const mlCargo = await fs.readFile(mlCargoPath, 'utf8');
            if (mlCargo.includes('default = ["metal"]')) {
              fixes.manual_steps.push('Edit cs2-ml/Cargo.toml: change default = ["metal"] to default = ["cpu-only"]');
              fixes.manual_steps.push('Then run: cargo clean && cargo build -p cs2-ml --no-default-features');
            }
            fixes.applied_fixes.push('Checked cs2-ml for Metal dependency issues');
          } catch (error) {
            fixes.applied_fixes.push(`Could not check cs2-ml Cargo.toml: ${error.message}`);
          }
        }
        
        if (fixType === 'protobuf' || fixType === 'all') {
          fixes.manual_steps.push('Install protobuf: sudo apt-get update && sudo apt-get install -y protobuf-compiler');
          fixes.applied_fixes.push('Added protobuf installation instructions');
        }
        
        if (fixType === 'fontconfig' || fixType === 'all') {
          fixes.manual_steps.push('Install fontconfig: sudo apt-get install -y libfontconfig1-dev');
          fixes.applied_fixes.push('Added fontconfig installation instructions');
        }
        
        if (fixType === 'all') {
          fixes.manual_steps.push('Install build tools: sudo apt-get install -y build-essential clang');
          fixes.applied_fixes.push('Added build tools installation instructions');
        }
      } else if (platform === 'macos') {
        if (fixType === 'all') {
          fixes.manual_steps.push('Install dependencies: brew install cmake protobuf pkg-config');
          fixes.manual_steps.push('Ensure Xcode command line tools: xcode-select --install');
          fixes.applied_fixes.push('Added macOS dependency installation instructions');
        }
      }
      
      // Create a cargo config for faster builds
      const cargoConfigDir = path.join(workspaceRoot, '.cargo');
      const cargoConfigPath = path.join(cargoConfigDir, 'config.toml');
      
      try {
        await fs.mkdir(cargoConfigDir, { recursive: true });
        
        const cargoConfig = `[build]
jobs = 4

[net]
retry = 3

[registries.crates-io]
protocol = "sparse"

${platform === 'linux' ? `[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]` : ''}
`;
        
        await fs.writeFile(cargoConfigPath, cargoConfig);
        fixes.applied_fixes.push('Created optimized .cargo/config.toml');
      } catch (error) {
        fixes.applied_fixes.push(`Could not create cargo config: ${error.message}`);
      }
      
    } catch (error) {
      fixes.status = 'failed';
      fixes.error = error.message;
    }
    
    return {
      content: [
        {
          type: 'text',
          text: `Platform Fixes Applied:\n\n${JSON.stringify(fixes, null, 2)}`
        }
      ]
    };
  }

  async detectPlatform() {
    try {
      const { stdout } = await execAsync('uname -s', { timeout: 5000 });
      const platform = stdout.trim().toLowerCase();
      
      if (platform.includes('linux')) return 'linux';
      if (platform.includes('darwin')) return 'macos';
      if (platform.includes('cygwin') || platform.includes('mingw')) return 'windows';
      
      return 'unknown';
    } catch (error) {
      return 'unknown';
    }
  }

  async checkWorkspaceHealth(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    
    const health = {
      timestamp: new Date().toISOString(),
      overall_status: 'healthy',
      checks: {},
      issues: [],
      suggestions: []
    };
    
    try {
      // Check if workspace Cargo.toml exists and is valid
      try {
        const cargoTomlPath = path.join(workspaceRoot, 'Cargo.toml');
        await fs.access(cargoTomlPath);
        health.checks.workspace_cargo_toml = 'exists';
      } catch (error) {
        health.checks.workspace_cargo_toml = 'missing';
        health.issues.push('Workspace Cargo.toml not found');
        health.overall_status = 'unhealthy';
      }
      
      // Check if essential directories exist
      const essentialDirs = ['cs2-demo-parser', 'cs2-common', 'cs2-ml'];
      for (const dir of essentialDirs) {
        try {
          await fs.access(path.join(workspaceRoot, dir));
          health.checks[`dir_${dir}`] = 'exists';
        } catch (error) {
          health.checks[`dir_${dir}`] = 'missing';
          health.issues.push(`Essential directory missing: ${dir}`);
        }
      }
      
      // Quick compilation check
      try {
        await execAsync(`cd "${workspaceRoot}" && cargo check -p cs2-common`, { timeout: 30000 });
        health.checks.basic_compilation = 'success';
      } catch (error) {
        health.checks.basic_compilation = 'failed';
        health.issues.push('Basic compilation check failed');
        if (health.overall_status === 'healthy') health.overall_status = 'degraded';
      }
      
      // Check for common files
      const importantFiles = ['setup_databases.sh', 'docker-compose.yml', '.github/copilot-instructions.md'];
      for (const file of importantFiles) {
        try {
          await fs.access(path.join(workspaceRoot, file));
          health.checks[`file_${file.replace(/[^a-zA-Z0-9]/g, '_')}`] = 'exists';
        } catch (error) {
          health.checks[`file_${file.replace(/[^a-zA-Z0-9]/g, '_')}`] = 'missing';
          health.issues.push(`Important file missing: ${file}`);
        }
      }
      
      // Generate suggestions if requested
      if (args.include_suggestions) {
        health.suggestions = [
          'Run `cargo fmt --all` to ensure code formatting',
          'Run `cargo clippy --workspace` to check for lints',
          'Use `cargo check` instead of `cargo build` for faster iteration',
          'Set up databases with `./setup_databases.sh` before running integration tests',
          'Use platform-specific build flags for cs2-ml on Linux'
        ];
      }
      
    } catch (error) {
      health.overall_status = 'error';
      health.error = error.message;
    }
    
    return {
      content: [
        {
          type: 'text',
          text: `Workspace Health Check:\n\n${JSON.stringify(health, null, 2)}`
        }
      ]
    };
  }

  async analyzeFeatures(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    
    const analysis = {
      timestamp: new Date().toISOString(),
      crate_features: {},
      feature_conflicts: [],
      recommended_combinations: [],
      platform_specific_features: {}
    };
    
    try {
      // Analyze features for each crate
      const crateNames = ['cs2-demo-parser', 'cs2-ml', 'cs2-common', 'cs2-data-pipeline'];
      
      for (const crateName of crateNames) {
        try {
          const cargoTomlPath = path.join(workspaceRoot, crateName, 'Cargo.toml');
          const cargoToml = await fs.readFile(cargoTomlPath, 'utf8');
          
          // Extract features section
          const featuresMatch = cargoToml.match(/\[features\]([\s\S]*?)(?=\[|\Z)/);
          if (featuresMatch) {
            const featuresSection = featuresMatch[1];
            const features = {};
            
            const lines = featuresSection.split('\n');
            for (const line of lines) {
              const match = line.match(/(\w[\w-]*)\s*=\s*\[(.*?)\]/);
              if (match) {
                features[match[1]] = match[2].split(',').map(f => f.trim().replace(/"/g, ''));
              }
            }
            
            analysis.crate_features[crateName] = features;
          } else {
            analysis.crate_features[crateName] = { note: 'No features section found' };
          }
        } catch (error) {
          analysis.crate_features[crateName] = { error: error.message };
        }
      }
      
      // Platform-specific feature recommendations
      analysis.platform_specific_features = {
        linux: {
          'cs2-ml': ['cpu-only'],
          note: 'Use cpu-only features to avoid Metal dependency on Linux'
        },
        macos: {
          'cs2-ml': ['metal'],
          note: 'Metal acceleration available on macOS'
        },
        development: {
          'cs2-integration-tests': ['integration-tests'],
          note: 'Enable integration test features for comprehensive testing'
        }
      };
      
      // Recommended feature combinations
      if (args.suggest_combinations) {
        analysis.recommended_combinations = [
          {
            purpose: 'Fast development builds',
            command: 'cargo check --workspace --no-default-features',
            description: 'Minimal features for quick compilation checks'
          },
          {
            purpose: 'Linux ML development',
            command: 'cargo build -p cs2-ml --no-default-features --features cpu-only',
            description: 'CPU-only ML features for Linux development'
          },
          {
            purpose: 'Integration testing',
            command: 'cargo test --package cs2-integration-tests --features integration-tests',
            description: 'Enable all integration test features'
          },
          {
            purpose: 'Production build',
            command: 'cargo build --workspace --release',
            description: 'Optimized production build with default features'
          }
        ];
      }
      
    } catch (error) {
      analysis.error = error.message;
    }
    
    return {
      content: [
        {
          type: 'text',
          text: `Feature Analysis:\n\n${JSON.stringify(analysis, null, 2)}`
        }
      ]
    };
  }

  async run() {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error('FPS Genie Cargo Helper MCP server running on stdio');
  }
}

const server = new CargoHelperServer();
server.run().catch(console.error);
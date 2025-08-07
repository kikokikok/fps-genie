#!/usr/bin/env node

/**
 * FPS Genie Demo Analyzer MCP Server
 * Provides CS2 demo file analysis and processing tools
 */

const { Server } = require('@modelcontextprotocol/sdk/server/index.js');
const { StdioServerTransport } = require('@modelcontextprotocol/sdk/server/stdio.js');
const { 
  CallToolRequestSchema,
  ListToolsRequestSchema,
} = require('@modelcontextprotocol/sdk/types.js');
const fs = require('fs').promises;
const path = require('path');
const { exec } = require('child_process');
const { promisify } = require('util');

const execAsync = promisify(exec);

class DemoAnalyzerServer {
  constructor() {
    this.server = new Server(
      {
        name: 'fps-genie-demo-analyzer',
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
          name: 'list_demo_files',
          description: 'List available CS2 demo files in the test_data directory',
          inputSchema: {
            type: 'object',
            properties: {
              directory: {
                type: 'string',
                description: 'Directory to search for demo files',
                default: 'test_data'
              },
              recursive: {
                type: 'boolean',
                description: 'Search recursively in subdirectories',
                default: false
              }
            }
          }
        },
        {
          name: 'analyze_demo_file',
          description: 'Analyze a CS2 demo file using the cs2-demo-parser',
          inputSchema: {
            type: 'object',
            properties: {
              demo_path: {
                type: 'string',
                description: 'Path to the demo file to analyze'
              },
              output_format: {
                type: 'string',
                enum: ['json', 'summary', 'detailed'],
                description: 'Output format for analysis results',
                default: 'summary'
              }
            },
            required: ['demo_path']
          }
        },
        {
          name: 'get_demo_metadata',
          description: 'Extract metadata from a CS2 demo file without full parsing',
          inputSchema: {
            type: 'object',
            properties: {
              demo_path: {
                type: 'string',
                description: 'Path to the demo file'
              }
            },
            required: ['demo_path']
          }
        },
        {
          name: 'run_demo_pipeline',
          description: 'Run the complete demo processing pipeline on a file',
          inputSchema: {
            type: 'object',
            properties: {
              demo_path: {
                type: 'string',
                description: 'Path to the demo file to process'
              },
              stages: {
                type: 'array',
                items: {
                  type: 'string',
                  enum: ['parse', 'analyze', 'store', 'vectorize']
                },
                description: 'Pipeline stages to execute',
                default: ['parse', 'analyze']
              }
            },
            required: ['demo_path']
          }
        },
        {
          name: 'validate_demo_file',
          description: 'Validate that a demo file is properly formatted and parseable',
          inputSchema: {
            type: 'object',
            properties: {
              demo_path: {
                type: 'string',
                description: 'Path to the demo file to validate'
              }
            },
            required: ['demo_path']
          }
        },
        {
          name: 'extract_key_moments',
          description: 'Extract key moments (clutches, aces, etc.) from a demo',
          inputSchema: {
            type: 'object',
            properties: {
              demo_path: {
                type: 'string',
                description: 'Path to the demo file'
              },
              moment_types: {
                type: 'array',
                items: {
                  type: 'string',
                  enum: ['ace', 'clutch', 'multi_kill', 'bomb_defuse', 'bomb_plant']
                },
                description: 'Types of key moments to extract',
                default: ['ace', 'clutch']
              }
            },
            required: ['demo_path']
          }
        },
        {
          name: 'compare_demo_performance',
          description: 'Compare performance metrics between multiple demo files',
          inputSchema: {
            type: 'object',
            properties: {
              demo_paths: {
                type: 'array',
                items: { type: 'string' },
                description: 'Array of demo file paths to compare'
              },
              metrics: {
                type: 'array',
                items: {
                  type: 'string',
                  enum: ['kills', 'deaths', 'assists', 'adr', 'rating', 'headshot_percentage']
                },
                description: 'Performance metrics to compare',
                default: ['kills', 'deaths', 'adr', 'rating']
              }
            },
            required: ['demo_paths']
          }
        }
      ]
    }));

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        switch (name) {
          case 'list_demo_files':
            return await this.listDemoFiles(args);
          case 'analyze_demo_file':
            return await this.analyzeDemoFile(args);
          case 'get_demo_metadata':
            return await this.getDemoMetadata(args);
          case 'run_demo_pipeline':
            return await this.runDemoPipeline(args);
          case 'validate_demo_file':
            return await this.validateDemoFile(args);
          case 'extract_key_moments':
            return await this.extractKeyMoments(args);
          case 'compare_demo_performance':
            return await this.compareDemoPerformance(args);
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

  async listDemoFiles(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    const searchDir = path.join(workspaceRoot, args.directory || 'test_data');
    
    try {
      const files = await this.findDemoFiles(searchDir, args.recursive);
      
      const fileInfo = await Promise.all(
        files.map(async (file) => {
          const stats = await fs.stat(file);
          return {
            path: path.relative(workspaceRoot, file),
            size: stats.size,
            modified: stats.mtime,
            readable: stats.isFile()
          };
        })
      );
      
      return {
        content: [
          {
            type: 'text',
            text: `Found ${fileInfo.length} demo files:\n\n${JSON.stringify(fileInfo, null, 2)}`
          }
        ]
      };
    } catch (error) {
      throw new Error(`Failed to list demo files: ${error.message}`);
    }
  }

  async findDemoFiles(dir, recursive = false) {
    const files = [];
    
    try {
      const entries = await fs.readdir(dir, { withFileTypes: true });
      
      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        
        if (entry.isDirectory() && recursive) {
          files.push(...await this.findDemoFiles(fullPath, recursive));
        } else if (entry.isFile() && entry.name.endsWith('.dem')) {
          files.push(fullPath);
        }
      }
    } catch (error) {
      // Directory might not exist, that's OK
    }
    
    return files;
  }

  async analyzeDemoFile(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    const demoPath = path.resolve(workspaceRoot, args.demo_path);
    
    // Validate file exists
    try {
      await fs.access(demoPath);
    } catch (error) {
      throw new Error(`Demo file not found: ${demoPath}`);
    }
    
    const outputFormat = args.output_format || 'summary';
    
    try {
      // Run the cs2-demo-parser
      const command = `cd "${workspaceRoot}" && cargo run -p cs2-demo-parser --bin parser -- "${demoPath}"`;
      const { stdout, stderr } = await execAsync(command, { 
        maxBuffer: 1024 * 1024 * 10, // 10MB buffer for large output
        timeout: 300000 // 5 minute timeout
      });
      
      let result;
      if (outputFormat === 'json') {
        // Try to parse as JSON if possible
        try {
          result = JSON.parse(stdout);
        } catch {
          result = { raw_output: stdout, stderr: stderr };
        }
      } else if (outputFormat === 'summary') {
        // Extract summary information
        result = this.extractSummary(stdout, stderr);
      } else {
        // Detailed format
        result = {
          full_output: stdout,
          errors: stderr,
          file_analyzed: demoPath,
          analysis_timestamp: new Date().toISOString()
        };
      }
      
      return {
        content: [
          {
            type: 'text',
            text: `Demo Analysis Results (${outputFormat}):\n\n${JSON.stringify(result, null, 2)}`
          }
        ]
      };
    } catch (error) {
      throw new Error(`Failed to analyze demo: ${error.message}`);
    }
  }

  extractSummary(stdout, stderr) {
    // Extract key information from parser output
    const summary = {
      status: stderr ? 'completed_with_warnings' : 'completed',
      file_size: 'unknown',
      parsing_duration: 'unknown',
      player_count: 'unknown',
      round_count: 'unknown',
      tick_count: 'unknown',
      warnings: stderr ? stderr.split('\n').filter(line => line.trim()) : [],
      key_stats: {}
    };
    
    // Try to extract common patterns from output
    const lines = stdout.split('\n');
    for (const line of lines) {
      if (line.includes('players:')) {
        const match = line.match(/(\d+)\s*players/);
        if (match) summary.player_count = parseInt(match[1]);
      }
      if (line.includes('rounds:')) {
        const match = line.match(/(\d+)\s*rounds/);
        if (match) summary.round_count = parseInt(match[1]);
      }
      if (line.includes('ticks:')) {
        const match = line.match(/(\d+)\s*ticks/);
        if (match) summary.tick_count = parseInt(match[1]);
      }
      if (line.includes('duration:') || line.includes('time:')) {
        const match = line.match(/(\d+(?:\.\d+)?)\s*(?:seconds?|mins?|minutes?)/);
        if (match) summary.parsing_duration = match[1];
      }
    }
    
    return summary;
  }

  async getDemoMetadata(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    const demoPath = path.resolve(workspaceRoot, args.demo_path);
    
    try {
      const stats = await fs.stat(demoPath);
      
      // Read first few bytes to get demo header info
      const buffer = Buffer.alloc(1024);
      const file = await fs.open(demoPath, 'r');
      await file.read(buffer, 0, 1024, 0);
      await file.close();
      
      const metadata = {
        file_path: path.relative(workspaceRoot, demoPath),
        file_size: stats.size,
        file_size_mb: (stats.size / (1024 * 1024)).toFixed(2),
        created: stats.birthtime,
        modified: stats.mtime,
        header_info: {
          magic: buffer.toString('ascii', 0, 8),
          // Add more header parsing as needed
        }
      };
      
      return {
        content: [
          {
            type: 'text',
            text: `Demo Metadata:\n\n${JSON.stringify(metadata, null, 2)}`
          }
        ]
      };
    } catch (error) {
      throw new Error(`Failed to get demo metadata: ${error.message}`);
    }
  }

  async runDemoPipeline(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    const stages = args.stages || ['parse', 'analyze'];
    
    const results = {
      demo_path: args.demo_path,
      stages_executed: [],
      stage_results: {},
      total_duration: 0,
      status: 'running'
    };
    
    const startTime = Date.now();
    
    try {
      for (const stage of stages) {
        const stageStart = Date.now();
        
        switch (stage) {
          case 'parse':
            results.stage_results.parse = await this.runParseStage(args.demo_path);
            break;
          case 'analyze':
            results.stage_results.analyze = await this.runAnalyzeStage(args.demo_path);
            break;
          case 'store':
            results.stage_results.store = await this.runStoreStage(args.demo_path);
            break;
          case 'vectorize':
            results.stage_results.vectorize = await this.runVectorizeStage(args.demo_path);
            break;
          default:
            throw new Error(`Unknown pipeline stage: ${stage}`);
        }
        
        const stageDuration = Date.now() - stageStart;
        results.stages_executed.push({
          stage,
          duration_ms: stageDuration,
          status: 'completed'
        });
      }
      
      results.total_duration = Date.now() - startTime;
      results.status = 'completed';
      
    } catch (error) {
      results.status = 'failed';
      results.error = error.message;
      results.total_duration = Date.now() - startTime;
    }
    
    return {
      content: [
        {
          type: 'text',
          text: `Demo Pipeline Results:\n\n${JSON.stringify(results, null, 2)}`
        }
      ]
    };
  }

  async runParseStage(demoPath) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    const command = `cd "${workspaceRoot}" && cargo run -p cs2-demo-parser --bin parser -- "${demoPath}"`;
    
    const { stdout, stderr } = await execAsync(command, { timeout: 300000 });
    return {
      stage: 'parse',
      stdout_lines: stdout.split('\n').length,
      has_errors: !!stderr,
      error_summary: stderr ? stderr.substring(0, 200) : null
    };
  }

  async runAnalyzeStage(demoPath) {
    // Placeholder for analysis stage
    return {
      stage: 'analyze',
      message: 'Analysis stage would extract key moments and statistics',
      implementation_needed: true
    };
  }

  async runStoreStage(demoPath) {
    // Placeholder for database storage stage
    return {
      stage: 'store',
      message: 'Storage stage would save data to PostgreSQL/TimescaleDB',
      implementation_needed: true
    };
  }

  async runVectorizeStage(demoPath) {
    // Placeholder for vectorization stage
    return {
      stage: 'vectorize', 
      message: 'Vectorization stage would create behavioral embeddings for Qdrant',
      implementation_needed: true
    };
  }

  async validateDemoFile(args) {
    const workspaceRoot = process.env.WORKSPACE_ROOT || '/home/runner/work/fps-genie/fps-genie';
    const demoPath = path.resolve(workspaceRoot, args.demo_path);
    
    const validation = {
      file_path: demoPath,
      is_valid: false,
      checks: {},
      errors: []
    };
    
    try {
      // Check if file exists
      await fs.access(demoPath);
      validation.checks.file_exists = true;
      
      // Check file size
      const stats = await fs.stat(demoPath);
      validation.checks.file_size = stats.size;
      validation.checks.file_size_ok = stats.size > 1024; // At least 1KB
      
      // Check file extension
      validation.checks.correct_extension = demoPath.endsWith('.dem');
      
      // Try to read header
      const buffer = Buffer.alloc(16);
      const file = await fs.open(demoPath, 'r');
      await file.read(buffer, 0, 16, 0);
      await file.close();
      
      validation.checks.readable = true;
      validation.checks.header_magic = buffer.toString('ascii', 0, 8);
      
      // Quick parse test
      try {
        const command = `cd "${workspaceRoot}" && timeout 30s cargo run -p cs2-demo-parser --bin parser -- "${demoPath}" 2>&1 | head -20`;
        const { stdout, stderr } = await execAsync(command);
        validation.checks.parseable = !stdout.includes('Error') && !stderr.includes('Error');
        validation.checks.parse_output = stdout.substring(0, 500);
      } catch (error) {
        validation.checks.parseable = false;
        validation.errors.push(`Parse test failed: ${error.message}`);
      }
      
      // Overall validation
      validation.is_valid = validation.checks.file_exists && 
                           validation.checks.file_size_ok && 
                           validation.checks.correct_extension && 
                           validation.checks.readable &&
                           validation.checks.parseable;
      
    } catch (error) {
      validation.errors.push(error.message);
    }
    
    return {
      content: [
        {
          type: 'text',
          text: `Demo File Validation:\n\n${JSON.stringify(validation, null, 2)}`
        }
      ]
    };
  }

  async extractKeyMoments(args) {
    // This would be implemented to identify key moments in demos
    const keyMoments = {
      demo_path: args.demo_path,
      moment_types: args.moment_types || ['ace', 'clutch'],
      extraction_status: 'placeholder_implementation',
      moments_found: [
        {
          type: 'ace',
          round: 5,
          tick: 45000,
          player: 'player_123',
          description: 'Player achieved ace (5 kills in round)'
        },
        {
          type: 'clutch',
          round: 12,
          tick: 89000,
          player: 'player_456',
          description: '1v3 clutch situation'
        }
      ],
      implementation_note: 'This is a placeholder. Real implementation would analyze demo data.'
    };
    
    return {
      content: [
        {
          type: 'text',
          text: `Key Moments Extraction:\n\n${JSON.stringify(keyMoments, null, 2)}`
        }
      ]
    };
  }

  async compareDemoPerformance(args) {
    // This would compare performance across multiple demos
    const comparison = {
      demo_paths: args.demo_paths,
      metrics: args.metrics || ['kills', 'deaths', 'adr', 'rating'],
      comparison_status: 'placeholder_implementation',
      performance_comparison: args.demo_paths.map((path, index) => ({
        demo: path,
        rank: index + 1,
        placeholder_stats: {
          kills: Math.floor(Math.random() * 30) + 10,
          deaths: Math.floor(Math.random() * 20) + 5,
          adr: Math.floor(Math.random() * 50) + 60,
          rating: (Math.random() * 1.5 + 0.5).toFixed(2)
        }
      })),
      implementation_note: 'This is a placeholder. Real implementation would analyze actual demo data.'
    };
    
    return {
      content: [
        {
          type: 'text',
          text: `Demo Performance Comparison:\n\n${JSON.stringify(comparison, null, 2)}`
        }
      ]
    };
  }

  async run() {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error('FPS Genie Demo Analyzer MCP server running on stdio');
  }
}

const server = new DemoAnalyzerServer();
server.run().catch(console.error);
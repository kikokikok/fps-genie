#!/usr/bin/env node

/**
 * FPS Genie Database Inspector MCP Server
 * Provides specialized database operations for the CS2 demo analysis system
 */

const { Server } = require('@modelcontextprotocol/sdk/server/index.js');
const { StdioServerTransport } = require('@modelcontextprotocol/sdk/server/stdio.js');
const { 
  CallToolRequestSchema,
  ListToolsRequestSchema,
} = require('@modelcontextprotocol/sdk/types.js');
const { Client } = require('pg');
const Redis = require('redis');

class DatabaseInspectorServer {
  constructor() {
    this.server = new Server(
      {
        name: 'fps-genie-database-inspector',
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
          name: 'inspect_demo_processing_status',
          description: 'Check the processing status of CS2 demo files',
          inputSchema: {
            type: 'object',
            properties: {
              demo_path: {
                type: 'string',
                description: 'Optional: specific demo file path to check'
              }
            }
          }
        },
        {
          name: 'query_player_snapshots',
          description: 'Query TimescaleDB for player snapshot data with time-series analysis',
          inputSchema: {
            type: 'object',
            properties: {
              match_id: {
                type: 'string',
                description: 'Match ID to query snapshots for'
              },
              player_id: {
                type: 'string', 
                description: 'Optional: specific player ID'
              },
              time_range: {
                type: 'string',
                description: 'Time range like "1 hour", "30 minutes", etc.'
              },
              limit: {
                type: 'number',
                description: 'Maximum number of snapshots to return',
                default: 100
              }
            },
            required: ['match_id']
          }
        },
        {
          name: 'search_behavioral_vectors', 
          description: 'Search Qdrant for similar behavioral patterns',
          inputSchema: {
            type: 'object',
            properties: {
              vector: {
                type: 'array',
                items: { type: 'number' },
                description: 'Behavioral vector to search for similar patterns'
              },
              collection: {
                type: 'string',
                description: 'Qdrant collection name',
                default: 'behavioral_vectors'
              },
              limit: {
                type: 'number', 
                description: 'Number of similar vectors to return',
                default: 10
              }
            },
            required: ['vector']
          }
        },
        {
          name: 'get_processing_queue_status',
          description: 'Check Redis processing queue status and job counts',
          inputSchema: {
            type: 'object',
            properties: {}
          }
        },
        {
          name: 'analyze_match_performance',
          description: 'Generate performance analysis for a specific match',
          inputSchema: {
            type: 'object',
            properties: {
              match_id: {
                type: 'string',
                description: 'Match ID to analyze'
              },
              analysis_type: {
                type: 'string',
                enum: ['heatmap', 'player_movement', 'tactical_analysis', 'key_moments'],
                description: 'Type of analysis to perform'
              }
            },
            required: ['match_id', 'analysis_type']
          }
        },
        {
          name: 'get_database_health',
          description: 'Check health and performance metrics of all databases',
          inputSchema: {
            type: 'object',
            properties: {}
          }
        }
      ]
    }));

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        switch (name) {
          case 'inspect_demo_processing_status':
            return await this.inspectDemoProcessingStatus(args);
          case 'query_player_snapshots':
            return await this.queryPlayerSnapshots(args);
          case 'search_behavioral_vectors':
            return await this.searchBehavioralVectors(args);
          case 'get_processing_queue_status':
            return await this.getProcessingQueueStatus(args);
          case 'analyze_match_performance':
            return await this.analyzeMatchPerformance(args);
          case 'get_database_health':
            return await this.getDatabaseHealth(args);
          default:
            throw new Error(`Unknown tool: ${name}`);
        }
      } catch (error) {
        return {
          content: [
            {
              type: 'text',
              text: `Error executing ${name}: ${error.message}`
            }
          ]
        };
      }
    });
  }

  async inspectDemoProcessingStatus(args) {
    const client = new Client({
      connectionString: process.env.DATABASE_URL
    });
    
    try {
      await client.connect();
      
      let query, params;
      if (args.demo_path) {
        query = `
          SELECT 
            demo_path,
            processed_at,
            processing_status,
            snapshot_count,
            processing_duration_seconds,
            error_message
          FROM demo_processing_status 
          WHERE demo_path = $1
        `;
        params = [args.demo_path];
      } else {
        query = `
          SELECT 
            demo_path,
            processed_at,
            processing_status,
            snapshot_count,
            processing_duration_seconds,
            COUNT(*) OVER() as total_demos,
            SUM(CASE WHEN processing_status = 'completed' THEN 1 ELSE 0 END) OVER() as completed_demos,
            SUM(CASE WHEN processing_status = 'failed' THEN 1 ELSE 0 END) OVER() as failed_demos
          FROM demo_processing_status 
          ORDER BY processed_at DESC 
          LIMIT 10
        `;
        params = [];
      }
      
      const result = await client.query(query, params);
      
      return {
        content: [
          {
            type: 'text',
            text: `Demo Processing Status:\n\n${JSON.stringify(result.rows, null, 2)}`
          }
        ]
      };
    } finally {
      await client.end();
    }
  }

  async queryPlayerSnapshots(args) {
    const client = new Client({
      connectionString: process.env.TIMESCALE_URL
    });
    
    try {
      await client.connect();
      
      const timeRange = args.time_range || '1 hour';
      const limit = args.limit || 100;
      
      let query = `
        SELECT 
          time,
          player_id,
          position_x,
          position_y,
          position_z,
          health,
          armor,
          weapon,
          velocity_x,
          velocity_y,
          velocity_z,
          view_angles_pitch,
          view_angles_yaw
        FROM player_snapshots 
        WHERE match_id = $1
      `;
      const params = [args.match_id];
      
      if (args.player_id) {
        query += ` AND player_id = $${params.length + 1}`;
        params.push(args.player_id);
      }
      
      query += ` AND time > NOW() - INTERVAL '${timeRange}'`;
      query += ` ORDER BY time DESC LIMIT $${params.length + 1}`;
      params.push(limit);
      
      const result = await client.query(query, params);
      
      return {
        content: [
          {
            type: 'text',
            text: `Player Snapshots (${result.rows.length} results):\n\n${JSON.stringify(result.rows, null, 2)}`
          }
        ]
      };
    } finally {
      await client.end();
    }
  }

  async searchBehavioralVectors(args) {
    const qdrantUrl = process.env.QDRANT_URL || 'http://localhost:6333';
    const collection = args.collection || 'behavioral_vectors';
    const limit = args.limit || 10;
    
    const response = await fetch(`${qdrantUrl}/collections/${collection}/points/search`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        vector: args.vector,
        limit: limit,
        with_payload: true,
        with_vector: false
      })
    });
    
    if (!response.ok) {
      throw new Error(`Qdrant search failed: ${response.statusText}`);
    }
    
    const result = await response.json();
    
    return {
      content: [
        {
          type: 'text',
          text: `Behavioral Vector Search Results:\n\n${JSON.stringify(result.result, null, 2)}`
        }
      ]
    };
  }

  async getProcessingQueueStatus(args) {
    const client = Redis.createClient({
      url: process.env.REDIS_URL || 'redis://localhost:6379'
    });
    
    try {
      await client.connect();
      
      const queueLength = await client.lLen('processing_queue');
      const processingCount = await client.lLen('currently_processing');
      const completedCount = await client.get('completed_count') || 0;
      const failedCount = await client.get('failed_count') || 0;
      
      const status = {
        queued_jobs: queueLength,
        currently_processing: processingCount,
        completed_jobs: parseInt(completedCount),
        failed_jobs: parseInt(failedCount),
        total_jobs: queueLength + processingCount + parseInt(completedCount) + parseInt(failedCount)
      };
      
      return {
        content: [
          {
            type: 'text',
            text: `Processing Queue Status:\n\n${JSON.stringify(status, null, 2)}`
          }
        ]
      };
    } finally {
      await client.disconnect();
    }
  }

  async analyzeMatchPerformance(args) {
    // This would implement complex match analysis queries
    const analysisResults = {
      match_id: args.match_id,
      analysis_type: args.analysis_type,
      placeholder: "Analysis implementation would query TimescaleDB for complex aggregations",
      suggestions: [
        "Query player_snapshots for movement patterns",
        "Aggregate key moments and clutch situations", 
        "Generate heatmap data from position coordinates",
        "Calculate player performance metrics"
      ]
    };
    
    return {
      content: [
        {
          type: 'text',
          text: `Match Performance Analysis:\n\n${JSON.stringify(analysisResults, null, 2)}`
        }
      ]
    };
  }

  async getDatabaseHealth(args) {
    const health = {
      timestamp: new Date().toISOString(),
      databases: {}
    };
    
    // Check PostgreSQL/TimescaleDB
    try {
      const client = new Client({
        connectionString: process.env.DATABASE_URL
      });
      await client.connect();
      
      const pgResult = await client.query('SELECT version(), now() as current_time');
      const timescaleResult = await client.query("SELECT extname, extversion FROM pg_extension WHERE extname = 'timescaledb'");
      
      health.databases.postgresql = {
        status: 'healthy',
        version: pgResult.rows[0].version,
        current_time: pgResult.rows[0].current_time,
        timescaledb_extension: timescaleResult.rows[0] || null
      };
      
      await client.end();
    } catch (error) {
      health.databases.postgresql = {
        status: 'error',
        error: error.message
      };
    }
    
    // Check Qdrant
    try {
      const qdrantUrl = process.env.QDRANT_URL || 'http://localhost:6333';
      const response = await fetch(`${qdrantUrl}/`);
      if (response.ok) {
        health.databases.qdrant = {
          status: 'healthy',
          url: qdrantUrl
        };
      } else {
        health.databases.qdrant = {
          status: 'error',
          error: `HTTP ${response.status}`
        };
      }
    } catch (error) {
      health.databases.qdrant = {
        status: 'error',
        error: error.message
      };
    }
    
    // Check Redis
    try {
      const client = Redis.createClient({
        url: process.env.REDIS_URL || 'redis://localhost:6379'
      });
      await client.connect();
      await client.ping();
      health.databases.redis = {
        status: 'healthy'
      };
      await client.disconnect();
    } catch (error) {
      health.databases.redis = {
        status: 'error',
        error: error.message
      };
    }
    
    return {
      content: [
        {
          type: 'text',
          text: `Database Health Check:\n\n${JSON.stringify(health, null, 2)}`
        }
      ]
    };
  }

  async run() {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error('FPS Genie Database Inspector MCP server running on stdio');
  }
}

const server = new DatabaseInspectorServer();
server.run().catch(console.error);
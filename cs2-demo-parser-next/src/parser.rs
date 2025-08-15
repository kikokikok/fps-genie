//! Parser Agent - Core Winnow-based parsing functionality
//!
//! This module implements the core parsing logic using Winnow parser combinators
//! for high-performance, composable parsing of CS2 demo files.

use winnow::prelude::*;
use winnow::{
    binary::{le_u32, le_u8},
    token::take,
    error::ErrMode,
};

type WinnowResult<T> = std::result::Result<T, winnow::error::ErrMode<winnow::error::ContextError>>;

use crate::common::{Error, Result, PerformanceMetrics};
use serde::{Deserialize, Serialize};

/// Demo file header information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoHeader {
    /// Demo file format identifier
    pub demo_file_stamp: String,
    
    /// Protocol version
    pub network_protocol: u32,
    
    /// Server name where demo was recorded
    pub server_name: String,
    
    /// Client name (player recording)
    pub client_name: String,
    
    /// Map name
    pub map_name: String,
    
    /// Game directory
    pub game_directory: String,
    
    /// Playback time in seconds
    pub playback_time: f32,
    
    /// Number of ticks in demo
    pub playback_ticks: u32,
    
    /// Number of frames in demo
    pub playback_frames: u32,
    
    /// Sign-on length
    pub signon_length: u32,
}

/// Demo command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoCommand {
    SignOn = 1,
    Packet = 2,
    SyncTick = 3,
    ConsoleCmd = 4,
    UserCmd = 5,
    DataTables = 6,
    Stop = 7,
    CustomData = 8,
    StringTables = 9,
}

impl TryFrom<u8> for DemoCommand {
    type Error = Error;
    
    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(DemoCommand::SignOn),
            2 => Ok(DemoCommand::Packet),
            3 => Ok(DemoCommand::SyncTick), 
            4 => Ok(DemoCommand::ConsoleCmd),
            5 => Ok(DemoCommand::UserCmd),
            6 => Ok(DemoCommand::DataTables),
            7 => Ok(DemoCommand::Stop),
            8 => Ok(DemoCommand::CustomData),
            9 => Ok(DemoCommand::StringTables),
            _ => Err(Error::InvalidFormat(format!("Unknown demo command: {}", value))),
        }
    }
}

/// Demo frame containing command and data
#[derive(Debug, Clone)]
pub struct DemoFrame {
    /// Command type
    pub command: DemoCommand,
    
    /// Tick number
    pub tick: u32,
    
    /// Player slot (for user commands)
    pub player_slot: Option<u8>,
    
    /// Frame data
    pub data: Vec<u8>,
}

/// Main demo parser using Winnow combinators
pub struct DemoParser {
    /// Current parsing position
    position: usize,
    
    /// Performance metrics
    metrics: PerformanceMetrics,
    
    /// Parsed header information
    header: Option<DemoHeader>,
}

impl DemoParser {
    /// Create a new demo parser instance
    pub fn new() -> Self {
        Self {
            position: 0,
            metrics: PerformanceMetrics::default(),
            header: None,
        }
    }
    
    /// Parse demo header using Winnow combinators
    pub fn parse_header(&mut self, data: &[u8]) -> Result<DemoHeader> {
        let start_time = std::time::Instant::now();
        
        // Parse the header using Winnow
        let mut input = &data[..];
        let header = demo_header_parser(&mut input)
            .map_err(|e| Error::Parse {
                message: format!("Header parsing failed: {}", e),
                position: self.position,
            })?;
            
        self.position = data.len() - input.len();
        self.header = Some(header.clone());
        
        // Update performance metrics
        self.metrics.parse_time_ms += start_time.elapsed().as_millis() as u64;
        self.metrics.bytes_processed += self.position;
        
        Ok(header)
    }
    
    /// Parse next frame from demo data
    pub fn parse_frame(&mut self, data: &[u8]) -> Result<Option<DemoFrame>> {
        if self.position >= data.len() {
            return Ok(None);
        }
        
        let start_time = std::time::Instant::now();
        
        let mut input = &data[self.position..];
        let result = demo_frame_parser(&mut input)
            .map_err(|e| Error::Parse {
                message: format!("Frame parsing failed: {}", e),
                position: self.position,
            });
            
        match result {
            Ok(frame) => {
                let bytes_consumed = data[self.position..].len() - input.len();
                self.position += bytes_consumed;
                
                // Update metrics
                self.metrics.parse_time_ms += start_time.elapsed().as_millis() as u64;
                self.metrics.bytes_processed += bytes_consumed;
                self.metrics.packets_parsed += 1;
                
                // Stop command indicates end of demo
                if frame.command == DemoCommand::Stop {
                    return Ok(None);
                }
                
                Ok(Some(frame))
            }
            Err(e) => Err(e),
        }
    }
    
    /// Get current parsing position
    pub fn position(&self) -> usize {
        self.position
    }
    
    /// Get performance metrics
    pub fn metrics(&mut self) -> &mut PerformanceMetrics {
        self.metrics.calculate_speed();
        &mut self.metrics
    }
    
    /// Get parsed header
    pub fn header(&self) -> Option<&DemoHeader> {
        self.header.as_ref()
    }
}

impl Default for DemoParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Winnow parser for demo file header
fn demo_header_parser(input: &mut &[u8]) -> WinnowResult<DemoHeader> {
    // Demo file starts with "HL2DEMO\0" signature
    let _signature: &[u8] = b"HL2DEMO\0".parse_next(input)?;
    
    // Demo file header protocol version
    let demo_protocol: u32 = le_u32.parse_next(input)?;
    let network_protocol: u32 = le_u32.parse_next(input)?;
    
    // Server and client names (null-terminated strings)
    let server_name = null_terminated_string.parse_next(input)?;
    let client_name = null_terminated_string.parse_next(input)?;
    let map_name = null_terminated_string.parse_next(input)?;
    let game_directory = null_terminated_string.parse_next(input)?;
    
    // Playback time and frame info
    let playback_time = le_f32.parse_next(input)?;
    let playback_ticks: u32 = le_u32.parse_next(input)?;
    let playback_frames: u32 = le_u32.parse_next(input)?;
    let signon_length: u32 = le_u32.parse_next(input)?;
    
    Ok(DemoHeader {
        demo_file_stamp: format!("HL2DEMO (protocol {})", demo_protocol),
        network_protocol,
        server_name,
        client_name,
        map_name,
        game_directory,
        playback_time,
        playback_ticks,
        playback_frames,
        signon_length,
    })
}

/// Winnow parser for demo frames
fn demo_frame_parser(input: &mut &[u8]) -> WinnowResult<DemoFrame> {
    // Read command type
    let cmd_byte: u8 = le_u8.parse_next(input)?;
    let command = DemoCommand::try_from(cmd_byte)
        .map_err(|_| ErrMode::Cut(winnow::error::ContextError::new()))?;
    
    // Read tick number
    let tick: u32 = le_u32.parse_next(input)?;
    
    // Player slot is only present for user commands
    let player_slot = if command == DemoCommand::UserCmd {
        Some(le_u8.parse_next(input)?)
    } else {
        None
    };
    
    // Read frame data length and data
    let data_length: u32 = le_u32.parse_next(input)?;
    let data: Vec<u8> = take(data_length as usize).parse_next(input)?.to_vec();
    
    Ok(DemoFrame {
        command,
        tick,
        player_slot,
        data,
    })
}

/// Parse null-terminated string
fn null_terminated_string(input: &mut &[u8]) -> WinnowResult<String> {
    let mut bytes = Vec::new();
    
    loop {
        let byte: u8 = le_u8.parse_next(input)?;
        if byte == 0 {
            break;
        }
        bytes.push(byte);
    }
    
    String::from_utf8(bytes)
        .map_err(|_| ErrMode::Cut(winnow::error::ContextError::new()))
}

/// Parse little-endian f32
fn le_f32(input: &mut &[u8]) -> WinnowResult<f32> {
    let bytes: &[u8] = take(4_usize).parse_next(input)?;
    Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demo_command_conversion() {
        assert_eq!(DemoCommand::try_from(1).unwrap(), DemoCommand::SignOn);
        assert_eq!(DemoCommand::try_from(7).unwrap(), DemoCommand::Stop);
        assert!(DemoCommand::try_from(255).is_err());
    }
    
    #[test]
    fn test_null_terminated_string_parser() {
        let data = b"hello\0world\0";
        let mut input = &data[..];
        
        let result = null_terminated_string(&mut input).unwrap();
        assert_eq!(result, "hello");
        
        let result2 = null_terminated_string(&mut input).unwrap();
        assert_eq!(result2, "world");
    }
    
    #[test]
    fn test_parser_metrics() {
        let mut parser = DemoParser::new();
        assert_eq!(parser.position(), 0);
        assert_eq!(parser.metrics().packets_parsed, 0);
    }
}
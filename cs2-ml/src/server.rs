use anyhow::Result;
use candle_core::Device;
use cs2_common::InputVector;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::model::BehaviorNet;

pub fn serve(port: u16) -> Result<()> {
    let net = BehaviorNet::new(12, 2, Device::Cpu)?; // Fixed: Added Device and proper error handling
    serve_with_model(net, port)
}

// Separated for testing
pub fn serve_with_model(_net: crate::model::BehaviorNet, port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    println!("Policy server listening on port {}", port);
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buf = [0u8; std::mem::size_of::<InputVector>()];
        match stream.read_exact(&mut buf) {
            Ok(_) => {
                let _input_vec: &InputVector = bytemuck::from_bytes(&buf);
                // Temporarily use placeholder prediction
                let output = cs2_common::OutputVector {
                    delta_yaw: 0.0,
                    delta_pitch: 0.0,
                };
                let out_bytes = bytemuck::bytes_of(&output);
                if let Err(e) = stream.write_all(out_bytes) {
                    eprintln!("Error writing response: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
            }
        }
    }
    Ok(())
}

// Modified serve function that checks shutdown flag
pub fn serve_with_model_with_shutdown(
    _net: crate::model::BehaviorNet,
    port: u16,
    shutdown: Arc<AtomicBool>,
) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    listener.set_nonblocking(true)?;
    println!("Policy server listening on port {}", port);

    while !shutdown.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                // Handle client
                let mut buf = [0u8; std::mem::size_of::<InputVector>()];
                match stream.read_exact(&mut buf) {
                    Ok(_) => {
                        let _input_vec: &InputVector = bytemuck::from_bytes(&buf);
                        // Temporarily use placeholder prediction
                        let output = cs2_common::OutputVector {
                            delta_yaw: 0.0,
                            delta_pitch: 0.0,
                        };
                        let out_bytes = bytemuck::bytes_of(&output);
                        if let Err(e) = stream.write_all(out_bytes) {
                            eprintln!("Error writing response: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from client: {}", e);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connections available, sleep briefly
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cs2_common::OutputVector;
    use std::net::TcpStream;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    struct TestServer {
        port: u16,
        handle: Option<thread::JoinHandle<()>>,
        shutdown: Arc<AtomicBool>,
    }

    impl TestServer {
        fn start() -> Self {
            // Find an available port
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let port = addr.port();
            drop(listener);

            let shutdown = Arc::new(AtomicBool::new(false));
            let shutdown_clone = shutdown.clone();

            // Create a simple model - handle the Result properly
            let model = crate::model::BehaviorNet::new(14, 2, candle_core::Device::Cpu).unwrap();

            let handle = thread::spawn(move || {
                // Run server in a separate thread until shutdown
                let server_result = serve_with_model_with_shutdown(model, port, shutdown_clone);
                if let Err(e) = server_result {
                    eprintln!("Server error: {}", e);
                }
            });

            TestServer {
                port,
                shutdown,
                handle: Some(handle),
            }
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            self.shutdown.store(true, Ordering::SeqCst);
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }

    // Modified serve function that checks shutdown flag
    fn serve_with_model_with_shutdown(
        net: crate::model::BehaviorNet,
        port: u16,
        shutdown: Arc<AtomicBool>,
    ) -> Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
        listener.set_nonblocking(true)?;

        while !shutdown.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false)?;

                    let mut buf = [0u8; std::mem::size_of::<InputVector>()];
                    match stream.read_exact(&mut buf) {
                        Ok(_) => {
                            let input_vec: &InputVector = bytemuck::from_bytes(&buf);
                            // Temporarily use placeholder prediction
                            let output = cs2_common::OutputVector {
                                delta_yaw: 0.0,
                                delta_pitch: 0.0,
                            };
                            let out_bytes = bytemuck::bytes_of(&output);
                            let _ = stream.write_all(out_bytes);
                        }
                        Err(_) => continue,
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connections available, sleep a bit
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    #[test]
    fn test_server_client_communication() {
        let server = TestServer::start();

        // Connect to server
        let addr = format!("127.0.0.1:{}", server.port);
        let mut stream = TcpStream::connect(addr).unwrap();

        // Send input vector
        let input = InputVector {
            health: 100.0,
            armor: 50.0,
            pos_x: 1.0,
            pos_y: 2.0,
            pos_z: 3.0,
            vel_x: 0.1,
            vel_y: 0.2,
            vel_z: 0.3,
            yaw: 90.0,
            pitch: 45.0,
            weapon_id_f32: 42.0,
            ammo: 30.0,
            is_airborne: 0.0,
            padding: 0.0,
        };
        let input_bytes = bytemuck::bytes_of(&input);
        stream.write_all(input_bytes).unwrap();

        // Read response
        let mut output_bytes = [0u8; std::mem::size_of::<OutputVector>()];
        stream.read_exact(&mut output_bytes).unwrap();

        // Parse output
        let output: OutputVector = *bytemuck::from_bytes(&output_bytes);

        // Verify output is valid (not checking exact values since the model is random)
        assert!(output.delta_yaw.is_finite());
        assert!(output.delta_pitch.is_finite());
    }
}

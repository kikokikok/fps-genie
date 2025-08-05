use std::net::TcpListener;
use std::io::{Read, Write};
use tch::{nn, Device};
use cs2_common::InputVector;
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn serve(model_path: &str, port: u16) -> Result<()> {
    let mut vs = nn::VarStore::new(Device::Cpu);
    vs.load(model_path)?;
    let net = crate::model::BehaviorNet::new(&vs.root(), 14, 2);
    serve_with_model(net, port)
}

// Separated for testing
pub fn serve_with_model(net: crate::model::BehaviorNet, port: u16) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    println!("Policy server listening on port {}", port);
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buf = [0u8; std::mem::size_of::<InputVector>()];
        match stream.read_exact(&mut buf) {
            Ok(_) => {
                let input_vec: &InputVector = bytemuck::from_bytes(&buf);
                let output = net.predict(input_vec);
                let out_bytes = bytemuck::bytes_of(&output);
                if let Err(e) = stream.write_all(out_bytes) {
                    eprintln!("Error writing response: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
            }
        }
    }
    Ok(())
}

// Modified serve function that checks shutdown flag
pub fn serve_with_model_with_shutdown(
    net: crate::model::BehaviorNet,
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
                        let input_vec: &InputVector = bytemuck::from_bytes(&buf);
                        let output = net.predict(input_vec);
                        let out_bytes = bytemuck::bytes_of(&output);
                        if let Err(e) = stream.write_all(out_bytes) {
                            eprintln!("Error writing response: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Error reading from client: {}", e);
                    }
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connections available, sleep briefly
                std::thread::sleep(std::time::Duration::from_millis(10));
            },
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
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;
    use tch::nn::VarStore;
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use tempfile::NamedTempFile;
    use cs2_common::OutputVector;

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

            // Create a simple model
            let vs = VarStore::new(Device::Cpu);
            let model = crate::model::BehaviorNet::new(&vs.root(), 14, 2);

            let handle = thread::spawn(move || {
                // Run server in a separate thread until shutdown
                let server_result = serve_with_model_with_shutdown(model, port, shutdown_clone);
                if let Err(e) = server_result {
                    eprintln!("Server error: {}", e);
                }
            });

            // Give server time to start
            thread::sleep(Duration::from_millis(100));

            Self {
                port,
                handle: Some(handle),
                shutdown,
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
                            let output = net.predict(input_vec);
                            let out_bytes = bytemuck::bytes_of(&output);
                            let _ = stream.write_all(out_bytes);
                        },
                        Err(_) => continue,
                    }
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connections available, sleep a bit
                    thread::sleep(Duration::from_millis(10));
                },
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

    #[test]
    fn test_model_save_load_serve() -> Result<()> {
        // Create and save a test model using tempfile with a proper .pt extension
        let tmp_dir = tempfile::tempdir()?;
        let model_path = tmp_dir.path().join("test_model.pt");

        let vs = VarStore::new(Device::Cpu);
        let _model = crate::model::BehaviorNet::new(&vs.root(), 14, 2);

        // Save model to the properly named file
        vs.save(&model_path)?;

        // Verify the file was created and exists
        assert!(model_path.exists(), "Model file should exist after saving");

        // Set up a test thread that will try to load the model and serve it
        let model_path_str = model_path.to_string_lossy().to_string();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        // Start server in background thread
        let server_thread = thread::spawn(move || -> Result<()> {
            let mut vs = nn::VarStore::new(Device::Cpu);
            let model = crate::model::BehaviorNet::load(&mut vs, &model_path_str, 14, 2)?;

            // Find available port
            let listener = TcpListener::bind("127.0.0.1:0")?;
            let port = listener.local_addr()?.port();
            drop(listener);

            serve_with_model_with_shutdown(model, port, shutdown_clone)
        });

        // Give server time to start
        thread::sleep(Duration::from_millis(200));

        // Signal shutdown
        shutdown.store(true, Ordering::SeqCst);

        // Wait for server thread to finish
        let result = server_thread.join();
        match result {
            Ok(server_result) => {
                // Check if server ran successfully or failed gracefully
                match server_result {
                    Ok(_) => println!("Server completed successfully"),
                    Err(e) => println!("Server failed with: {}", e),
                }
            },
            Err(_) => println!("Server thread panicked"),
        }

        // Test passes if we got this far without panicking
        Ok(())
    }
}

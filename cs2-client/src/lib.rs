use anyhow::Result;
use cs2_common::{CS2Error, InputVector, OutputVector};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

/// A client for connecting to the CS2 ML policy server
pub struct PolicyClient {
    connection: TcpStream,
}

impl PolicyClient {
    /// Connect to a policy server at the given address
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        let connection = TcpStream::connect(addr).map_err(|e| {
            CS2Error::NetworkError(format!("Failed to connect to policy server: {}", e))
        })?;

        // Set non-blocking mode
        connection.set_nonblocking(false)?;

        Ok(Self { connection })
    }

    /// Get a policy prediction for the given input
    pub fn predict(&mut self, input: &InputVector) -> Result<OutputVector> {
        // Convert to bytes and send
        let input_bytes: &[u8] = bytemuck::bytes_of(input);
        self.connection
            .write_all(input_bytes)
            .map_err(|e| CS2Error::NetworkError(format!("Failed to send input: {}", e)))?;

        // Read response
        let mut output_bytes = [0u8; std::mem::size_of::<OutputVector>()];
        self.connection
            .read_exact(&mut output_bytes)
            .map_err(|e| CS2Error::NetworkError(format!("Failed to read prediction: {}", e)))?;

        // Convert back to OutputVector
        let output = bytemuck::pod_read_unaligned::<OutputVector>(&output_bytes);
        Ok(output)
    }
}

/// A higher-level interface for game integration
pub struct AIController {
    client: PolicyClient,
}

impl AIController {
    /// Create a new AI controller connected to a policy server
    pub fn new(server_addr: impl ToSocketAddrs) -> Result<Self> {
        let client = PolicyClient::connect(server_addr)?;
        Ok(Self { client })
    }

    /// Get aim adjustment based on current game state
    pub fn get_aim_adjustment(&mut self, state: &PlayerState) -> Result<(f32, f32)> {
        let input = InputVector {
            health: state.health,
            armor: state.armor,
            pos_x: state.position.0,
            pos_y: state.position.1,
            pos_z: state.position.2,
            vel_x: state.velocity.0,
            vel_y: state.velocity.1,
            vel_z: state.velocity.2,
            yaw: state.view_angles.0,
            pitch: state.view_angles.1,
            weapon_id_f32: state.weapon_id as f32,
            ammo: state.ammo,
            is_airborne: if state.is_airborne { 1.0 } else { 0.0 },
            padding: 0.0,
        };

        let output = self.client.predict(&input)?;
        Ok((output.delta_yaw, output.delta_pitch))
    }
}

/// Struct to encapsulate player state for aim adjustment
pub struct PlayerState {
    pub health: f32,
    pub armor: f32,
    pub position: (f32, f32, f32),
    pub velocity: (f32, f32, f32),
    pub view_angles: (f32, f32),
    pub weapon_id: u16,
    pub ammo: f32,
    pub is_airborne: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cs2_common::{InputVector, OutputVector};
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    // A mock policy server for testing
    struct MockPolicyServer {
        addr: SocketAddr,
        thread_handle: Option<thread::JoinHandle<()>>,
        shutdown: Arc<Mutex<bool>>,
    }

    impl MockPolicyServer {
        fn start() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let shutdown = Arc::new(Mutex::new(false));
            let shutdown_clone = shutdown.clone();

            let handle = thread::spawn(move || {
                listener.set_nonblocking(true).unwrap();

                while !*shutdown_clone.lock().unwrap() {
                    match listener.accept() {
                        Ok((mut socket, _)) => {
                            socket.set_nonblocking(false).unwrap();

                            // Handle this client in a new thread
                            thread::spawn(move || {
                                loop {
                                    let mut buf = [0u8; std::mem::size_of::<InputVector>()];
                                    match socket.read_exact(&mut buf) {
                                        Ok(_) => {
                                            // Simulate policy server - always return a fixed response
                                            let response = OutputVector {
                                                delta_yaw: 1.0,
                                                delta_pitch: 0.5,
                                            };
                                            let response_bytes = bytemuck::bytes_of(&response);
                                            if socket.write_all(response_bytes).is_err() {
                                                break;
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                            });
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(_) => break,
                    }
                }
            });

            // Give the server a moment to start
            thread::sleep(Duration::from_millis(50));

            Self {
                addr,
                thread_handle: Some(handle),
                shutdown,
            }
        }
    }

    impl Drop for MockPolicyServer {
        fn drop(&mut self) {
            *self.shutdown.lock().unwrap() = true;
            if let Some(handle) = self.thread_handle.take() {
                let _ = handle.join();
            }
        }
    }

    #[test]
    fn test_client_connection_with_mock_server() {
        let server = MockPolicyServer::start();
        let client = PolicyClient::connect(server.addr);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_prediction() {
        let server = MockPolicyServer::start();
        let mut client = PolicyClient::connect(server.addr).unwrap();

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

        let output = client.predict(&input).unwrap();
        assert_eq!(output.delta_yaw, 1.0);
        assert_eq!(output.delta_pitch, 0.5);
    }

    #[test]
    fn test_ai_controller_integration() {
        let server = MockPolicyServer::start();
        let mut controller = AIController::new(server.addr).unwrap();

        let state = PlayerState {
            health: 100.0,
            armor: 0.0,
            position: (100.0, 200.0, 50.0),
            velocity: (0.0, 0.0, 0.0),
            view_angles: (45.0, 30.0),
            weapon_id: 1,
            ammo: 30.0,
            is_airborne: false,
        };

        let result = controller.get_aim_adjustment(&state);

        assert!(result.is_ok());
        let (delta_yaw, delta_pitch) = result.unwrap();
        assert_eq!(delta_yaw, 1.0);
        assert_eq!(delta_pitch, 0.5);
    }

    // Integration test with testcontainers
    // This is commented out because it requires Docker and would be run in CI
    /*
    #[test]
    #[cfg(feature = "integration-tests")]
    fn test_with_real_policy_server() {
        use testcontainers::{clients, Container, Docker, Image};

        struct PolicyServerImage;

        impl Image for PolicyServerImage {
            type Args = ();

            fn name(&self) -> String {
                "cs2-ml-policy".to_string()
            }

            fn tag(&self) -> String {
                "latest".to_string()
            }

            fn ready_conditions(&self) -> Vec<testcontainers::core::ReadyCondition> {
                vec![testcontainers::core::ReadyCondition::message_on_stderr("Policy server listening on port")]
            }
        }

        let docker = clients::Cli::default();
        let container: Container<PolicyServerImage> = docker.run(PolicyServerImage);
        let port = container.get_host_port_ipv4(8123);
        let addr = format!("127.0.0.1:{}", port);

        let client = PolicyClient::connect(addr).unwrap();
        // Run tests with the real policy server...
    }
    */

    #[test]
    #[ignore] // Requires a running server
    fn test_real_server_connection() {
        let client = PolicyClient::connect("127.0.0.1:8123");
        assert!(client.is_ok());
    }
}

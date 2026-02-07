use crate::models::{ChatRequest, LlamaCppConfig, ModelInfo};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
pub struct LlamaServer;

impl LlamaServer {
    pub async fn spawn(
        model_entry: Option<ModelInfo>,
        config: LlamaCppConfig,
        client: reqwest::Client,
    ) -> Result<(u16, Child), String> {
        println!("Starting model with config: {:?}", config);
        let model_path = if let Some(entry) = model_entry {
            if let Some(path) = &entry.model_file_path {
                PathBuf::from(path)
            } else {
                PathBuf::from(&config.model_path)
            }
        } else {
            PathBuf::from(&config.model_path)
        };

        if !model_path.exists() {
            return Err(format!("Model file not found: {:?}", model_path));
        }

        let mut llama_server_path = PathBuf::from(&config.llama_cpp_path);
        let is_exec = llama_server_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_lowercase().contains("llama-server"))
            .unwrap_or(false);

        if !is_exec || llama_server_path.is_dir() {
            let candidates = if cfg!(windows) {
                vec!["llama-server.exe", "llama-server.cmd", "llama-server"]
            } else {
                vec!["llama-server"]
            };
            let mut found = false;
            for candidate in &candidates {
                let p = llama_server_path.join(candidate);
                if p.exists() {
                    llama_server_path = p;
                    found = true;
                    break;
                }
            }

            if !found && llama_server_path.is_dir() {
                let build_candidates = if cfg!(windows) {
                    vec![
                        "build/bin/Release/llama-server.exe",
                        "build/bin/Release/llama-server.cmd",
                        "bin/llama-server.exe",
                        "bin/llama-server.cmd",
                    ]
                } else {
                    vec!["build/bin/Release/llama-server", "bin/llama-server"]
                };
                for candidate in &build_candidates {
                    let p = llama_server_path.join(candidate);
                    if p.exists() {
                        llama_server_path = p;
                        found = true;
                        break;
                    }
                }
            }

            if !found && !is_exec {
                let suffix = if cfg!(windows) {
                    "llama-server.exe"
                } else {
                    "llama-server"
                };
                llama_server_path = llama_server_path.join(suffix);
            }
        }

        if !llama_server_path.exists() {
            return Err(format!(
                "llama-server executable not found: {:?}",
                llama_server_path
            ));
        }

        println!(
            "[Infrastructure] Spawning llama-server at: {:?} with port {}",
            llama_server_path, config.port
        );
        let binary_dir = llama_server_path.parent().unwrap_or(Path::new("."));
        let mut cmd = Command::new(&llama_server_path);
        #[cfg(windows)]
        {
            // Prevents a console window from flashing when starting the server.
            cmd.creation_flags(0x08000000);
        }
        let mut child = cmd
            .current_dir(binary_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-m")
            .arg(&model_path)
            .arg("--port")
            .arg(config.port.to_string())
            .arg("-c")
            .arg(config.ctx_size.to_string())
            .arg("-np")
            .arg(config.parallel.to_string())
            .arg("-ngl")
            .arg(config.n_gpu_layers.to_string())
            .spawn()
            .map_err(|e| format!("Failed to spawn llama-server: {}", e))?;

        Self::pipe_output(&mut child);

        // Healthcheck
        let port = config.port;
        let health_url = format!("http://localhost:{}/health", port);
        let mut attempts = 0;
        let max_attempts = if cfg!(test) { 2 } else { 40 };
        let sleep_duration = if cfg!(test) {
            Duration::from_millis(10)
        } else {
            Duration::from_millis(500)
        };

        while attempts < max_attempts {
            if let Ok(Some(status)) = child.try_wait() {
                return Err(format!("llama-server exited early with status: {}", status));
            }

            if let Ok(res) = client
                .get(&health_url)
                .timeout(Duration::from_secs(1))
                .send()
                .await
            {
                if res.status().is_success() {
                    return Ok((port, child));
                }
            }
            tokio::time::sleep(sleep_duration).await;
            attempts += 1;
        }

        let _ = child.kill().await;
        Err("Failed to start model: Healthcheck timed out".to_string())
    }

    fn pipe_output(child: &mut Child) {
        if let Some(stdout) = child.stdout.take() {
            let mut reader = tokio::io::BufReader::new(stdout);
            tauri::async_runtime::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break;
                    }
                    print!("[llama-server] {}", line);
                    line.clear();
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let mut reader = tokio::io::BufReader::new(stderr);
            tauri::async_runtime::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break;
                    }
                    eprint!("[llama-server] {}", line);
                    line.clear();
                }
            });
        }
    }

    pub async fn stream_chat(
        client: reqwest::Client,
        port: u16,
        request: ChatRequest,
    ) -> Result<mpsc::Receiver<String>, String> {
        let url = format!("http://localhost:{}/v1/chat/completions", port);
        let (tx, rx) = mpsc::channel(32);

        tauri::async_runtime::spawn(async move {
            let res = client
                .post(&url)
                .json(&request)
                .timeout(Duration::from_secs(300))
                .send()
                .await;
            match res {
                Ok(mut response) => {
                    if !response.status().is_success() {
                        let _ = tx
                            .send(format!("Error: Status {}", response.status()))
                            .await;
                        return;
                    }
                    let mut buffer = String::new();
                    while let Ok(Some(chunk)) = response.chunk().await {
                        let s = String::from_utf8_lossy(&chunk);
                        buffer.push_str(&s);
                        while let Some(idx) = buffer.find('\n') {
                            let line = buffer[..idx].trim().to_string();
                            buffer = buffer[idx + 1..].to_string();
                            if line.is_empty() || line.starts_with(':') {
                                continue;
                            }

                            if line.starts_with("data:") {
                                let data_content = line["data:".len()..].trim();
                                if data_content == "[DONE]" {
                                    return;
                                }
                                if let Ok(json) =
                                    serde_json::from_str::<serde_json::Value>(data_content)
                                {
                                    if let Some(content) =
                                        json["choices"][0]["delta"]["content"].as_str()
                                    {
                                        if tx.send(content.to_string()).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            } else {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                                    if let Some(content) =
                                        json["choices"][0]["delta"]["content"].as_str()
                                    {
                                        if tx.send(content.to_string()).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("Error: Connect {}", e)).await;
                }
            }
        });
        Ok(rx)
    }
}

impl LlamaServer {
    pub fn test_pipe_output(child: &mut Child) {
        Self::pipe_output(child);
    }
}

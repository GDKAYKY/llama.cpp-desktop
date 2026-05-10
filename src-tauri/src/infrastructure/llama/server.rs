use crate::models::{ChatRequest, LlamaCppConfig, ModelInfo};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
pub struct LlamaServer;

fn ensure_non_empty_input(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{} cannot be empty", label));
    }

    Ok(())
}

fn validate_existing_file_with_extension(
    value: impl AsRef<Path>,
    label: &str,
    expected_extensions: &[&str],
) -> Result<PathBuf, String> {
    let path = value.as_ref();
    let value = path.to_string_lossy();
    ensure_non_empty_input(value.as_ref(), label)?;

    let path = path.to_path_buf();
    if !path.exists() {
        return Err(format!("{} not found: {}", label, path.display()));
    }

    if !path.is_file() {
        return Err(format!("{} must be a file: {}", label, path.display()));
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| {
            let extensions = expected_extensions
                .iter()
                .map(|ext| format!(".{}", ext.trim_start_matches('.')))
                .collect::<Vec<_>>()
                .join(" or ");
            format!("{} must have an extension of {}", label, extensions)
        })?;

    if expected_extensions
        .iter()
        .any(|expected| extension.eq_ignore_ascii_case(expected.trim_start_matches('.')))
    {
        Ok(path)
    } else {
        let extensions = expected_extensions
            .iter()
            .map(|ext| format!(".{}", ext.trim_start_matches('.')))
            .collect::<Vec<_>>()
            .join(" or ");
        Err(format!(
            "{} must have an extension of {}",
            label, extensions
        ))
    }
}

fn validate_existing_file(value: impl AsRef<Path>, label: &str) -> Result<PathBuf, String> {
    let path = value.as_ref();
    let value = path.to_string_lossy();
    ensure_non_empty_input(value.as_ref(), label)?;

    let path = path.to_path_buf();
    if !path.exists() {
        return Err(format!("{} not found: {}", label, path.display()));
    }

    if !path.is_file() {
        return Err(format!("{} must be a file: {}", label, path.display()));
    }

    Ok(path)
}

fn validate_binary_path(value: &str) -> Result<PathBuf, String> {
    ensure_non_empty_input(value, "binary_path")?;

    let path = PathBuf::from(value);
    if !path.exists() {
        return Err(format!("binary_path not found: {}", path.display()));
    }

    if path.is_file() && cfg!(windows) {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                format!(
                    "binary_path must point to a .exe, .cmd, or .bat file when using a file path: {}",
                    path.display()
                )
            })?;

        if !["exe", "cmd", "bat"]
            .iter()
            .any(|expected| extension.eq_ignore_ascii_case(expected))
        {
            return Err(format!(
                "binary_path must point to a .exe, .cmd, or .bat file when using a file path: {}",
                path.display()
            ));
        }
    }

    Ok(path)
}

fn validate_inline_chat_template(template: &str) -> Result<(), String> {
    ensure_non_empty_input(template, "chat_template")
}

fn matches_managed_flag(arg: &str, flag: &str) -> bool {
    arg == flag
        || arg
            .strip_prefix(flag)
            .is_some_and(|suffix| suffix.starts_with('='))
}

fn validate_extra_args(extra_args: &[String]) -> Result<(), String> {
    const MANAGED_FLAGS: &[&str] = &[
        "-m",
        "--model",
        "--port",
        "-c",
        "--ctx-size",
        "-np",
        "--parallel",
        "-ngl",
        "--gpu-layers",
        "--n-gpu-layers",
        "--jinja",
        "--chat-template",
        "--chat-template-file",
    ];

    for arg in extra_args {
        ensure_non_empty_input(arg, "extra_args entry")?;

        if let Some(flag) = MANAGED_FLAGS
            .iter()
            .copied()
            .find(|flag| matches_managed_flag(arg, flag))
        {
            return Err(format!(
                "extra_args cannot override managed flag {}. Update the corresponding app setting instead.",
                flag
            ));
        }
    }

    Ok(())
}

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

        validate_existing_file(&model_path, "model_path")?;
        let mut llama_server_path = validate_binary_path(&config.llama_cpp_path)?;
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

        if config.chat_template.is_some() && config.chat_template_file.is_some() {
            return Err("Use either chat_template or chat_template_file, not both".to_string());
        }

        if let Some(template) = &config.chat_template {
            validate_inline_chat_template(template)?;
        }

        if let Some(template_file) = &config.chat_template_file {
            validate_existing_file_with_extension(template_file, "chat_template_file", &["jinja"])?;
        }

        validate_extra_args(&config.extra_args)?;

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
        cmd.current_dir(binary_dir)
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
            .arg(config.n_gpu_layers.to_string());

        if config.jinja {
            cmd.arg("--jinja");
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        if let Some(template) = &config.chat_template {
            cmd.arg("--chat-template").arg(template);
        }

        if let Some(template_file) = &config.chat_template_file {
            cmd.arg("--chat-template-file").arg(template_file);
        }

        let mut child = cmd
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
                        if buffer.trim() == "data: [DONE]" {
                            return;
                        }

                        while let Some(idx) = buffer.find("\n\n") {
                            let block = buffer[..idx].to_string();
                            buffer = buffer[idx + 2..].to_string();

                            for raw_line in block.lines() {
                                let line = raw_line.trim();
                                if line.is_empty() || line.starts_with(':') {
                                    continue;
                                }
                                let payload = if line.starts_with("data:") {
                                    line["data:".len()..].trim()
                                } else {
                                    line
                                };

                                if payload == "[DONE]" {
                                    return;
                                }

                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload)
                                {
                                    let chunks = extract_stream_chunks(&json);
                                    if !chunks.is_empty() {
                                        for chunk in chunks {
                                            if tx.send(chunk).await.is_err() {
                                                return;
                                            }
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

    pub async fn chat_completion(
        client: reqwest::Client,
        port: u16,
        request: ChatRequest,
    ) -> Result<serde_json::Value, String> {
        let url = format!("http://localhost:{}/v1/chat/completions", port);
        let response = client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if body.is_empty() {
                return Err(format!("Chat completion failed: {}", status));
            }
            return Err(format!("Chat completion failed: {} - {}", status, body));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
}

impl LlamaServer {
    pub fn test_pipe_output(child: &mut Child) {
        Self::pipe_output(child);
    }
}

fn extract_stream_chunks(json: &serde_json::Value) -> Vec<String> {
    let mut chunks = Vec::new();

    if let Some(reasoning) = json["choices"][0]["delta"]["reasoning_content"].as_str() {
        if !reasoning.is_empty() {
            chunks.push(format!("<think>{}</think>", reasoning));
        }
    }

    if let Some(reasoning) = json["choices"][0]["message"]["reasoning_content"].as_str() {
        if !reasoning.is_empty() {
            chunks.push(format!("<think>{}</think>", reasoning));
        }
    }

    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
        if !content.is_empty() {
            chunks.push(content.to_string());
        }
    }

    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
        if !content.is_empty() {
            chunks.push(content.to_string());
        }
    }

    if let Some(content) = json["choices"][0]["delta"]["text"].as_str() {
        if !content.is_empty() {
            chunks.push(content.to_string());
        }
    }

    chunks
}

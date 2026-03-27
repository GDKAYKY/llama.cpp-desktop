use tauri::AppHandle;
use tauri::Manager;
use std::path::PathBuf;
use reqwest::Client;
use serde_json::Value;

/// Garante que o chat template Jinja de um repo HuggingFace esteja em cache.
pub async fn ensure_hf_chat_template(
    app: &AppHandle,
    hf_repo: &str,
    revision: Option<&str>,
) -> Result<PathBuf, String> {
    // 1. Resolve o diretório de cache: <app_data>/chat_templates/
    let cache_dir: PathBuf = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Erro ao obter diretório de dados: {}", e))?
        .join("chat_templates");

    // 2. Cria o diretório de cache caso ainda não exista
    tokio::fs::create_dir_all(&cache_dir)
        .await
        .map_err(|e| format!("Erro ao criar diretório de cache: {}", e))?;

    // 3. Gera o nome de arquivo seguro: '/' e ':' viram '_'
    let slug = hf_repo.replace('/', "_").replace(':', "_");
    let cache_path = cache_dir.join(format!("{}.jinja", slug));

    // 4. Cache hit → retorna imediatamente sem fazer requisição
    if cache_path.exists() {
        return Ok(cache_path);
    }

    // 5. Cache miss → tenta baixar o arquivo .jinja direto do HuggingFace
    let client = reqwest::Client::builder()
        .user_agent("llama-desktop/1.0")
        .build()
        .map_err(|e| format!("Erro ao criar cliente HTTP: {}", e))?;

    let template_content = download_chat_template(&client, hf_repo, revision.unwrap_or("main")).await?;

    // 6. Salva o conteúdo no cache
    tokio::fs::write(&cache_path, template_content)
        .await
        .map_err(|e| format!("Erro ao salvar template em cache: {}", e))?;

    Ok(cache_path)
}

/// Tenta baixar o chat template em ordem: jinja file -> tokenizer_config.json
async fn download_chat_template(client: &Client, hf_repo: &str, revision: &str) -> Result<String, String> {
    // Tenta primeiro o arquivo .jinja direto
    let url_jinja = format!(
        "https://huggingface.co/{}/resolve/{}/chat_template.jinja",
        hf_repo,
        revision
    );

    let res = client.get(&url_jinja).send().await;
    if let Ok(response) = res {
        if response.status().is_success() {
            return response
                .text()
                .await
                .map_err(|e| format!("Erro ao ler corpo da resposta .jinja: {}", e));
        }
    }

    // Fallback: Tenta extrair do tokenizer_config.json
    let url_config = format!(
        "https://huggingface.co/{}/resolve/{}/tokenizer_config.json",
        hf_repo,
        revision
    );

    let response = client
        .get(&url_config)
        .send()
        .await
        .map_err(|e| format!("Erro ao buscar tokenizer_config.json: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Não foi possível encontrar chat_template.jinja nem tokenizer_config.json (Status: {})",
            response.status()
        ));
    }

    let config: Value = response
        .json()
        .await
        .map_err(|e| format!("Erro ao parsear tokenizer_config.json: {}", e))?;

    let template = config["chat_template"]
        .as_str()
        .ok_or_else(|| "Campo 'chat_template' não encontrado em tokenizer_config.json".to_string())?;

    Ok(template.to_string())
}

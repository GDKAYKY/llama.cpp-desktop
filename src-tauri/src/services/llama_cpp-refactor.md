# Actor-style LLM Orchestrator – Instructions

Este documento define **todos os passos, decisões arquiteturais e regras** para refatorar o serviço atual de `llama.cpp` para um **Actor-style service**, alinhado a um app desktop (Tauri) que orquestra múltiplos modelos locais definidos dinamicamente via JSON.

---

## 1. Objetivo

Criar um **orquestrador robusto de LLMs locais** que:

* Não usa `Arc<Mutex<>>`
* Possui **um único dono de estado** (actor)
* Orquestra processos externos (`llama-server`)
* Trabalha com **modelos definidos dinamicamente** via registry JSON
* Suporta **streaming** sem race conditions
* É adequado para **desktop app** (UI concorrente)

---

## 2. Princípios não negociáveis

1. **Actor é o dono absoluto do estado**
2. Nenhuma função pública muta estado diretamente
3. Nenhum `sleep()` para sincronização
4. Nenhum enum hardcoded de modelos
5. Nenhum cosplay de OpenAI (`gpt-3.5-turbo`, etc.)
6. Streaming é responsabilidade interna do actor

---

## 3. Fonte da verdade dos modelos

Os modelos **NÃO** são definidos em código.

Eles vêm **exclusivamente** do JSON no formato:

* `provider`
* `library`
* `name`
* `version`
* `manifest`
* `model_file_path`
* `full_identifier`

👉 **`full_identifier` é o ModelId real.**

---

## 4. Identidade do Modelo

```text
ModelId = full_identifier (string única e estável)
```

Representação em Rust:

* `ModelId(String)`
* Deve implementar `Clone + Eq + Hash`

Nenhum enum de modelos é permitido.

---

## 5. Estruturas de dados base

### 5.1 Registry

* O JSON é carregado no boot da aplicação
* Convertido para uma estrutura interna

Responsabilidade:

* Descrever modelos disponíveis
* NÃO gerenciar processo

---

### 5.2 Estado interno do Actor

Para cada modelo:

* Metadados (vindos do JSON)
* Porta alocada
* Status atual
* Processo (`Child`)

Estados possíveis:

* `Stopped`
* `Starting`
* `Running`
* `Crashed(reason)`

---

## 6. Actor: responsabilidade e escopo

O Actor:

* Vive em **uma task Tokio dedicada**
* Recebe comandos via `mpsc`
* Responde via `oneshot` ou stream (`mpsc`)

Ele é responsável por:

* Start / Stop de modelos
* Healthcheck
* Spawn de processos
* Streaming SSE
* Garantir consistência de estado

---

## 7. Mensagens do Actor

O actor aceita **somente mensagens**, nunca chamadas diretas.

Tipos de comando:

* Start de modelo
* Stop de modelo
* Envio de chat
* Consulta de status
* Listagem de modelos

Cada comando:

* Deve ser idempotente quando possível
* Deve responder explicitamente sucesso ou erro

---

## 8. Fluxo de inicialização

1. App inicia
2. JSON de modelos é carregado
3. Actor é criado com o registry
4. Actor entra em loop aguardando comandos
5. Nenhum modelo é iniciado automaticamente (a menos que explicitado)

---

## 9. Start de modelo (passo a passo)

1. Receber comando `Start(model_id)`
2. Validar existência no registry
3. Verificar se já está `Running` ou `Starting`
4. Resolver caminho do binário `llama-server`
5. Resolver caminho do modelo (`model_file_path`)
6. Alocar porta (fixa ou dinâmica)
7. Spawn do processo
8. Atualizar status para `Starting`
9. Poll de readiness (`/health` ou `/v1/models`)
10. Se ok → `Running`
11. Se falhar → `Crashed`

⚠️ **Proibido usar `sleep()` fixo**

---

## 10. Stop de modelo

1. Receber comando `Stop(model_id)`
2. Se não estiver rodando → noop
3. Enviar `kill()` ao processo
4. Aguardar `wait()` com timeout
5. Limpar processo
6. Atualizar status para `Stopped`

---

## 11. Envio de Chat

1. Receber `SendChat(model_id, request)`
2. Verificar status == `Running`
3. Se não → erro imediato
4. Criar task de streaming
5. Conectar ao servidor local do modelo
6. Parsear SSE corretamente (`data:` + `\n\n`)
7. Enviar chunks via `mpsc`
8. Encerrar stream ao receber `[DONE]`

---

## 12. Streaming (regras)

* Streaming é sempre assíncrono
* Nunca bloqueia o actor
* SSE não pode assumir alinhamento por linha
* Bufferizar até evento completo

---

## 13. Status e observabilidade

* `Status(model_id)` **NÃO** pode mutar estado
* Falha de processo deve atualizar status para `Crashed`
* Logs de stdout/stderr devem ser capturados (ou opcionalmente habilitados)

---

## 14. Integração com Tauri

* Tauri chama comandos → envia mensagem ao actor
* Actor responde via `oneshot` ou stream
* UI nunca acessa estado direto
* Actor sobrevive a múltiplos invokes concorrentes

---

## 15. O que foi explicitamente evitado

* `Arc<Mutex<>>`
* State machine espalhada
* Enum hardcoded de modelos
* `sleep()` mágico
* API OpenAI fake
* Funções com efeitos colaterais ocultos

---

## 16. Resultado esperado

Ao final:

* Código previsível
* Concorrência segura
* Fácil de testar
* Fácil de debugar
* Escalável para novos modelos

Este design é **adequado para produção desktop**, não demo.

---

## 17. Próximo passo

Após este documento:

* Implementar o actor
* Refatorar o serviço atual para mensagens
* Conectar UI
* Adicionar testes de stress

Fim das instruções.

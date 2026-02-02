# Llama.cpp Desktop

Uma aplicação desktop moderna construída com Tauri para executar e gerenciar modelos Llama.cpp localmente com interface premium.

## 🎯 Objetivo

Fornecer uma interface gráfica intuitiva e poderosa para interagir com modelos de linguagem Llama.cpp, eliminando a necessidade de usar linha de comando e oferecendo uma experiência de usuário superior.

## 🏗️ Arquitetura

### Frontend
- **Framework**: Svelte 5 com Vite
- **Interface**: UI moderna e responsiva
- **Comunicação**: APIs Tauri para integração com backend

### Backend
- **Runtime**: Rust com Tauri v2
- **Integração**: Llama.cpp nativo para máxima performance
- **Padrões**: Modelos centralizados em `src-tauri/src/models/`

## 🚀 Funcionalidades

- ✅ Execução local de modelos Llama.cpp
- ✅ Interface gráfica intuitiva
- ✅ Gerenciamento de modelos
- ✅ Performance otimizada
- ✅ Multiplataforma (Windows, macOS, Linux)

## 📁 Estrutura do Projeto

```
llama.cpp-desktop/
├── src/                    # Frontend Svelte
├── src-tauri/             # Backend Rust
│   └── src/models/        # Modelos compartilhados (padrão obrigatório)
├── docs/                  # Documentação
│   ├── PROJECT_STRUCTURE.md
│   ├── BACKEND_ARCHITECTURE.md
│   └── MODELS_SETUP_GUIDE.md
└── README.md
```

## 🛠️ Desenvolvimento

### Pré-requisitos
- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)

### Comandos
```bash
# Instalar dependências
npm install

# Executar em modo desenvolvimento
npm run dev

# Build para produção
npm run build
```

## 📖 Documentação

- **[Estrutura do Projeto](./docs/PROJECT_STRUCTURE.md)**: Visão detalhada da organização
- **[Padrões de Arquitetura](./docs/BACKEND_ARCHITECTURE.md)**: Standards obrigatórios do backend Rust
- **[Guia de Configuração](./docs/MODELS_SETUP_GUIDE.md)**: Como configurar e executar modelos

## 🎨 Tecnologias

| Categoria | Tecnologia |
|-----------|------------|
| Frontend | Svelte 5, Vite, TypeScript |
| Backend | Rust, Tauri v2 |
| AI/ML | Llama.cpp |
| Build | Vite, Cargo |

## 🔧 Padrões de Desenvolvimento

- **Modelos Centralizados**: Todos os modelos Rust compartilhados devem estar em `src-tauri/src/models/`
- **Arquitetura Limpa**: Separação clara entre frontend e backend
- **Performance First**: Otimização para execução local eficiente

## 🚀 Roadmap

- [ ] Suporte a mais formatos de modelo
- [ ] Interface de chat aprimorada
- [ ] Configurações avançadas de modelo
- [ ] Exportação de conversas
- [ ] Temas personalizáveis

---

*Desenvolvido com ❤️ pela equipe Llama Desktop*

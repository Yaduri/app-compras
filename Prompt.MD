Atue como um Engenheiro de Software Sênior, Especialista em DevOps e Arquiteto Rust.
Seu objetivo é preparar o ambiente de desenvolvimento local do zero, estruturar a arquitetura e implementar um aplicativo multiplataforma de Lista de Compras Colaborativa (focado em Android e Desktop/Web, compartilhando o mesmo código e interface).

**1. Visão Geral do Produto**
Um aplicativo de lista de compras inteligente e colaborativo, pensado para casais e famílias. O foco principal é permitir que múltiplos usuários gerenciem e utilizem a mesma lista simultaneamente no supermercado, com sincronização em tempo real e foco em usabilidade prática com apenas uma mão.

**2. Restrição Orçamentária e Stack Tecnológica (100% Gratuita)**
Toda a arquitetura deve utilizar ferramentas open-source e serviços em nuvem com planos gratuitos permanentes (Free Tier):
* **Linguagem Principal:** Rust (estruturado em Cargo Workspace para compartilhar tipos e modelos entre front e back).
* **Frontend / Mobile / Desktop:** Dioxus (utilizando Signals para gerenciamento de estado reativo e compilando para Android e PC/Web com a mesma UI).
* **Estilização:** TailwindCSS integrado ao Dioxus ou CSS modular leve focado em mobile-first.
* **Backend / API:** Axum + Tokio + Tower (preparado para deploy gratuito no Fly.io ou Render).
* **Banco de Dados:** Supabase (PostgreSQL no Free Tier) utilizando `sqlx` no backend.
* **Autenticação:** Supabase Auth suportando Login com Google (OAuth2) e E-mail/Senha (JWT validado no Axum).
* **Tempo Real (Real-time):** WebSockets nativos via Axum (usando `tokio::sync::broadcast`) ou integração com Supabase Realtime para refletir alterações na lista em milissegundos.

**3. Funcionalidades Essenciais (Escopo do Sistema)**
* **Autenticação de Usuários:** Cadastro e login via E-mail/Senha ou conta Google.
* **Vínculo de Contas por Código de Convite:** Cada lista criada gera um código alfanumérico curto e único (ex: `CASA-8F2A`). Outros usuários podem inserir esse código no app para entrar na lista, ganhando permissão para adicionar, editar, marcar e excluir itens em conjunto.
* **Sincronização em Tempo Real:** Quando um usuário marca um item como "comprado" ou adiciona um novo produto, a tela de todos os membros vinculados àquela lista deve atualizar instantaneamente sem precisar recarregar.
* **Categorização Automática:** Os itens devem ser agrupados automaticamente por seções do mercado (Hortifruti, Açougue, Limpeza, Laticínios, Padaria, Mercearia, Outros) com base em um dicionário de produtos conhecidos, permitindo alteração manual.
* **Modo Supermercado (UI Focada):** Um modo de visualização otimizado para o momento da compra, com fontes maiores, alto contraste e checkboxes grandes (fácil uso com uma mão). Itens marcados vão automaticamente para o final da lista.
* **Cálculo de Orçamento e Total Parcial:** Campos opcionais para quantidade e preço unitário em cada item. O app deve exibir o "Total Planejado" e o "Total no Carrinho" (soma apenas dos itens já marcados) em tempo real.
* **Histórico e Sugestões Inteligentes:** O banco de dados deve registrar a frequência de compra dos itens e o último preço pago, oferecendo uma aba de "Mais Comprados" para adicionar itens recorrentes com apenas um toque.
* **Identificação de Ação:** Exibir discretamente na interface quem adicionou ou quem marcou determinado item da lista.

**4. Arquitetura do Projeto (Cargo Workspace)**
O projeto será um monorepo Rust com 3 crates principais:
1. `shared`: Structs serializáveis (`serde`), DTOs de requisição/resposta, enums de categorias e payloads de eventos WebSocket compartilhados entre o app e a API.
2. `backend`: Servidor Axum com rotas REST (autenticação, criação de listas, geração/validação de códigos de convite, CRUD de itens), handler de WebSockets para atualização em tempo real e conexão com PostgreSQL via `sqlx`.
3. `frontend`: Aplicativo Dioxus estruturado em componentes reutilizáveis, gerenciamento de estado reativo, cliente HTTP (`reqwest`) e cliente WebSocket para manter a lista sincronizada.

**5. FASE 0: Verificação e Instalação do Ambiente no PC (Execute Primeiro)**
Assuma que o computador pode não ter o Rust ou as dependências instaladas. Antes de criar qualquer arquivo de código, use o terminal para verificar e instalar tudo o que for necessário:
1. **Verifique o Sistema e Pré-requisitos C/C++:**
   * Identifique o sistema operacional. Se for Windows, verifique se o compilador/linker MSVC (`link.exe`) está presente. Caso não esteja, instale o Visual Studio Build Tools via `winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"` (ou oriente/instale a toolchain GNU caso necessário).
   * Verifique se o `git` está instalado; se não estiver, instale-o.
2. **Verifique e Instale o Rust (`rustup`, `cargo`, `rustc`):**
   * Rode `rustc --version` e `cargo --version`.
   * Se não estiverem instalados, baixe e instale o `rustup` automaticamente via terminal (no Windows via `winget install Rustlang.Rustup` ou baixando o `rustup-init.exe` com `-y`; no Linux/macOS via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`).
   * **Importante:** Após instalar, recarregue as variáveis de ambiente (`PATH` / `$env:Path`) na sessão atual do terminal adicionando `~/.cargo/bin` (ou `%USERPROFILE%\.cargo\bin`) para que os comandos `cargo` e `rustup` funcionem imediatamente sem precisar reiniciar o editor.
3. **Targets e Ferramentas do Ecossistema Rust:**
   * Adicione o target WebAssembly para testes rápidos no PC: `rustup target add wasm32-unknown-unknown`.
   * Adicione os targets do Android para futura compilação mobile: `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`.
   * Instale o `cargo-binstall` (para acelerar instalações de binários em Rust) e em seguida instale o CLI do Dioxus (`dioxus-cli`, comando `dx`).
4. **Verificação de Android SDK/NDK (Não-bloqueante):**
   * Verifique se variáveis como `ANDROID_HOME` e `ANDROID_NDK_HOME` (e o Java JDK) existem na máquina. Registre o status delas e documente no arquivo `ARCHITECTURE.md` exatamente o que falta ou os comandos para instalar o Android SDK/NDK quando formos gerar o APK na Fase 3.

**6. FASE 1: Estruturação do Workspace e Arquitetura (Execute Após a Fase 0)**
Assim que o `cargo` estiver funcionando perfeitamente no terminal, execute a Fase 1 diretamente no diretório do projeto:
1. Crie um arquivo `ARCHITECTURE.md` na raiz contendo:
   * O roadmap completo dividido em 4 fases (Fase 1: Setup, Workspace e Contratos; Fase 2: Backend Axum, Auth e WebSockets; Fase 3: App Dioxus Base, Telas e Vínculo por Código; Fase 4: Tempo Real, Modo Supermercado, Orçamento e Histórico).
   * A modelagem SQL completa (`users`, `lists`, `list_members`, `categories`, `items`, `item_history`).
   * Os contratos das rotas HTTP REST e dos eventos WebSocket.
   * O diagnóstico do ambiente local e instruções para o build Android.
2. Crie um arquivo `migrations/001_initial_schema.sql` com o script SQL completo (incluindo chaves estrangeiras, índices para busca rápida por código de convite e triggers se necessário) pronto para rodar no Supabase.
3. Inicialize o Cargo Workspace criando o `Cargo.toml` na raiz e os diretórios `shared`, `backend` e `frontend` com seus respectivos `Cargo.toml` e arquivos iniciais (`src/lib.rs` e `src/main.rs`).
4. Implemente de forma completa no crate `shared` todas as structs de domínio, enums de categorias com lógica de categorização automática por palavras-chave, DTOs de autenticação/listas e payloads de mensagens WebSocket.
5. Crie um arquivo `.env.example` documentando as variáveis necessárias (`DATABASE_URL`, `SUPABASE_URL`, `SUPABASE_ANON_KEY`, `SUPABASE_JWT_SECRET`, `PORT`).

**7. Regras de Compilação e Boas Práticas**
* No `backend`, utilize funções `sqlx::query` / `sqlx::query_as` em vez das macros `sqlx::query!`, para permitir que o projeto compile com `cargo check` mesmo sem o banco de dados estar conectado em tempo de compilação.
* Não tente compilar o APK Android (`dx build --platform android`) nesta primeira etapa.
* Ao finalizar a criação dos arquivos da Fase 1, execute `cargo check --workspace` (e `cargo test --package shared` se criar testes unitários para a categorização automática) para validar que todo o código compila sem erros.
* Quando o `cargo check --workspace` passar com sucesso, apresente um resumo do que foi instalado e criado, e aguarde minha confirmação para iniciar a **Fase 2 (Backend Axum)**.
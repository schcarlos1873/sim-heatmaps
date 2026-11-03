---
name: sim-heatmaps
description: >
  Leia esta skill em conjunto com qualquer outra skill do projeto sim-heatmaps. Contém
  convenções de arquitetura, padrões de nomenclatura SQLite, estrutura de
  projetos. Use sempre que for criar qualquer artefato
  (entidade, endpoint, componente) no projeto sim-heatmaps.
---


## Pré-Requisitos

- O programa deve ser multiplataforma ou seja excutar em Linux, MacOS e Windows.
- Deve permitir multipos idiomas em pt-BR e us-EN
- Sempre sugira opções as solicitações
- Trabalhe sempre no diretório do projeto
- Crie uma pasta ./tmp para os arquivos que serão gerados pelo Claude.
- Sempre questione quando houver dúvidas.
- Execute somente quando for autorizado
- Permitir temas White / Black


## Arquitetura

Seguir as boas práticas de arquitetura limpa, separando as camadas de domínio, aplicação e infraestrutura. O backend é organizado em projetos distintos para cada camada:

- Arquitetura em Camadas

    Domain (Domínio) - Regras de negócio centrais
    Application (Aplicação) - Casos de uso e orquestração
    Infrastructure (Infraestrutura) - Acesso a dados e implementações
    Camada de Apresentação native webview usando Tauri


- Principais Características
1. Domain Layer:

    Entidades Machine (máquina de papel), Property (propriedade + limites) e Sample (amostra medida)
    Repository pattern com traits (interfaces) definidas no domínio
    Domain Services para regras complexas (estatísticas, classificação por limites, geração .al3)
    Validações de negócio (código de propriedade único por máquina, ordem dos limites, Sample ID obrigatório, data válida)

2. Application Layer:

    Application Services para casos de uso
    Mapeamento entre entidades e DTOs se houver necessidade
    Tratamento de exceções de negócio, gerar logs de erros na pasta /log

3. Infrastructure Layer:

    **rusqlite** (feature `bundled`) para persistência SQLite — não usar .NET/Entity Framework
    Migrações SQL versionadas executadas na inicialização (`PRAGMA user_version`)
    Implementação concreta dos repositórios (traits do domínio)
    Escritor de arquivos `.al3` (UTF-8 sem BOM, quebras de linha CRLF)

4. Apresentacão(UI) Layer:
   - Cores modernas foco  azul
   - Estilo visual conforme a skill `sim-heatmaps-style` (paleta neutra, accent azul)

### Estrutura de pastas

```
sim-heatmaps/
├── src-tauri/                     # Workspace Cargo
│   ├── Cargo.toml                 # [workspace] members = crates/*
│   ├── crates/
│   │   ├── domain/                # Entidades, value objects, traits de repositório, regras
│   │   ├── application/           # Casos de uso (services), DTOs, erros, logging em /log
│   │   └── infrastructure/        # rusqlite, migrações, repositórios, escritor .al3
│   └── src/                       # App Tauri: main.rs + commands (apenas adaptação para a UI)
├── ui/                            # Vite + React + TypeScript + Tailwind
│   └── src/
│       ├── i18n/                  # pt-BR.json, en-US.json
│       ├── theme/                 # tokens CSS (claro/escuro/sistema)
│       ├── features/              # properties, machines, samples, settings
│       └── components/            # componentes reutilizáveis
├── tmp/                           # Scripts Python gerados pelo Claude (stdlib, --dry-run)
├── log/                           # Logs de erro da aplicação
├── documents/                     # Manual Autoline 600 (referência)
└── export-files/                  # Arquivos .al3 originais (referência de formato)
```

Regra de dependência: `src (Tauri) → application → domain` e `infrastructure → domain`.
O domínio não depende de nenhuma outra camada nem de crates de I/O.

### Limites das propriedades

Cada propriedade possui os limites `Target`, `SpecHigh`, `WarningHigh`, `WarningLow` e `SpecLow`,
armazenados no SQLite. Ordem obrigatória:

`SpecLow < WarningLow < Target < WarningHigh < SpecHigh`

Cores (baseadas no aviso de limite da grade do manual Autoline 600):

| Faixa do valor                              | Cor      |
|---------------------------------------------|----------|
| Entre WarningLow e WarningHigh               | verde (ok) |
| Entre Warning e Spec                         | amarelo (aviso) |
| Fora de SpecLow / SpecHigh                   | vermelho (fora de especificação) |



## Stack 

- Liguagem de programação **"Rust Lang"**
- Banco de dados **"Sqlite"**
- Arquivos de exportação no format **"UTF-8"**
- Frontend Tauri + Vite + React 
- Estilo **"Tailwind CSS"**

.



## Banco de dados
-
### Padrões de Colunas


## Scripts e Automação

- **Linguagem padrão para scripts:** Python 3 (`.py`)
- Scripts ficam em `.claude/{skill}/scaffold/`
- Todo script deve suportar `--dry-run` para visualizar saída sem efeitos colaterais
- Usar somente bibliotecas da stdlib do Python (sem dependências externas)
- Argumentos via `argparse`; cores via sequências ANSI inline (sem `rich` ou similares)
### Armazenamento

- Banco e logs ficam em locais graváveis do sistema e **não são configuráveis** pelo usuário:
  banco em `<app data>/sim-heatmaps.db`; logs em `./log` (dev) ou `<app data>/log` (produção).
- Somente a pasta de exportação `.al3` é configurável (tabela `settings`).

### Formato `.al3` (deduzido de `export-files/`)

- UTF-8 sem BOM, CRLF, termina com `end AL300 SAMPLE\r\n`; cabeçalho fixo de 26 linhas.
- `PROP-SUMMARY`: `cod;cod2;nome;NA;NA;média;desvio;n;mín;máx;0;NA;NA;n` — desvio padrão **populacional**, média/desvio com 2 casas, sem zeros à direita.
- Cada propriedade tem **grade de posições própria** (o módulo define: ex. MB6 Gloss 160→4000, Tear 480→4320, Taber 320→4160, cor em 2560). Ponto único usa `Position 1;valor`.
- Nomes são preservados literalmente (há nomes com espaço final, ex.: `Indice de rasgo L `).
- Validação: `python3 tmp/validate_al3.py --strict-stats <arquivos>`.

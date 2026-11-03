---
name: sim-heatmaps-style
description: Desenvolver ou adaptar interfaces de aplicativos com cores neutras, em temas claro e escuro. Use quando o usuário solicitar esse estilo visual; não aplica uma interface de chat sem que ela faça parte do produto.
---

# Aplicativos com estilo

Desenvolva o aplicativo solicitado com superfícies neutras, hierarquia discreta, tipografia legível e cores de destaque contidas. A paleta abaixo é uma proposta de inspiração visual, não uma especificação oficial ou reprodução exata do ChatGPT.

## Escopo e implementação

- Identifique o objetivo do aplicativo e inspecione a estrutura e o sistema visual existentes antes de editar. Preserve a tecnologia e as convenções do projeto; em um projeto novo, use a plataforma pedida e escolha uma solução simples quando ela não tiver sido especificada.
- Implemente os fluxos solicitados. Não acrescente chat, integração com IA, autenticação ou serviços pagos apenas por causa da referência visual.
- Centralize as cores em tokens semânticos. Use variáveis CSS em aplicações web e o mecanismo equivalente de tema em outras plataformas. Evite cores literais espalhadas pelos componentes.
- Adapte a paleta à identidade já definida e a escolhas explícitas do usuário. Não utilize nome, logotipo ou outros elementos de marca do ChatGPT como identidade do produto, salvo solicitação específica.

## Paleta de referência

| Token | Tema claro | Tema escuro | Uso |
|---|---|---|---|
| background | #FFFFFF | #212121 | Fundo principal |
| sidebar | #F9F9F9 | #171717 | Navegação lateral |
| surface | #F4F4F4 | #2F2F2F | Campos e cartões |
| surface-hover | #EAEAEA | #3A3A3A | Interação sobre superfícies |
| text-primary | #171717 | #ECECEC | Texto principal |
| text-secondary | #595959 | #B4B4B4 | Descrições e legendas |
| border | #D9D9D9 | #484848 | Separadores decorativos |
| control-border | #767676 | #8A8A8A | Contorno quando necessário para identificar um controle |
| accent | #0F6B55 | #6DD6B4 | Links, foco e pequenos destaques |
| on-accent | #FFFFFF | #10271F | Conteúdo sobre accent |
| primary-action | #171717 | #ECECEC | Botão principal |
| on-primary-action | #FFFFFF | #171717 | Conteúdo do botão principal |
| danger | #B42318 | #FFB4AB | Erros e ações destrutivas |

Prefira botões principais monocromáticos; reserve verde para destaques pontuais. Não use verde como preenchimento de grandes áreas por padrão. Verifique o contraste das combinações realmente usadas, inclusive estados de hover e foco; tokens isolados não garantem acessibilidade.

## Composição e comportamento

- Use fonte de sistema sans-serif ou a fonte já adotada pelo produto, com texto base próximo de 16 px na web. Trabalhe com espaçamentos consistentes em múltiplos de 4 ou 8 px.
- Prefira superfícies planas, bordas discretas e cantos entre 8 e 16 px. Reserve sombras suaves para menus e elementos sobrepostos. Evite gradientes e efeitos decorativos sem função.
- Organize conteúdo de leitura em largura confortável. Em aplicativos com navegação lateral, recolha-a em telas pequenas e mantenha ações essenciais acessíveis.
- Forneça estados de hover, foco visível, seleção, desabilitado, carregamento, vazio e erro nos componentes que os exigirem. Use texto ou ícones junto à cor para comunicar estados.
- Garanta navegação por teclado, rótulos acessíveis e alvos de toque adequados. Na web, busque contraste mínimo de 4,5:1 para texto comum e 3:1 para texto grande e indicadores essenciais de controles.
- Em projetos novos sem preferência de tema, acompanhe o tema do sistema e permita escolha persistida se houver configurações de aparência. Não substitua uma preferência de tema já estabelecida.

## Verificação e entrega

Verifique os fluxos alterados, a adaptação a telas pequenas, os dois temas quando implementados e a interação por teclado. Utilize as verificações existentes do projeto e inspecione a interface renderizada quando houver ferramenta disponível. Se não for possível inspecionar visualmente, informe essa limitação sem afirmar que houve validação visual.

Ao entregar, resuma o que foi implementado, as verificações realizadas e como executar ou visualizar o resultado. Distinga funcionalidades reais de dados demonstrativos ou integrações ainda não configuradas.

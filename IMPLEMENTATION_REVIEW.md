# BiGameMode - Revisão e Sugestões de Melhoria (UX/UI, Código, Funcionalidades)

Durante a exploração do código da interface em GTK4 (`bigame-ui`) e o núcleo de orquestração (`bigame-core`), montei o seguinte relatório técnico sugerindo melhorias substanciais ao aplicativo em múltiplos eixos:

## 1. UX/UI e Design (Libadwaita/GTK4)
- **Estados Vazios (Empty States):** Como o BiGameMode roda em `systemctl` (via o daemon `falcond`), se ele detectar que o serviço ou permissões do PolicyKit falharam, a UI atual exibe erros brutos. Melhoraria imensamente a UX ao adotar o componente `adw::StatusPage` para guiar a correção interativa da falta de permissão ou falta de pacotes.
- **Microinterações:** Durante a ativação de um modo denso de performance, usar botões que exibem "loading" temporários ajudaria em respostas mais fluídas se a comunicação D-Bus ou backend demorasse 300-500ms.
- **Feedback Constante (Branding):** Em relatórios de "Sistema Não Baseado em BigLinux/TigerOS", seria viável utilizar `adw::Banner` (toast messages dinâmicos em vez de Dialogs agressivos modais).
- **Ícones Simbólicos Contextuais:** Refinar os `.ui` (ou layouts programáticos) para usar classes `.suggested-action` e `.destructive-action` em todos os perfis predefinidos que aplicam escalonamento alto de CPU.

## 2. Refatoração de Código e Arquitetura (Rust)
- **Desacoplamento Assíncrono (async/await):** Dentro do `bigame-ui`, operações do GTK não devem ser bloqueadas. Identifiquei algumas áreas que esperam respostas síncronas bloqueando Main Thread. Utilizar o `glib::MainContext::spawn_local` junto das primitivas de `tokio` melhorará a responsividade quando o DBUS do `falcond` é acionado para aplicar os perfis (especialmente se I/O pesado ou systemd estão envolvidos na resposta).
- **Despacho Total via DBus:** Ao invés do binário gráfico subornar o shell com `sudo -n /usr/bin/mkdir` na hora de salvar propriedades, a interface `bigame-ui` deveria enviar um objeto JSON pelo D-Bus para o `falcond` (Que roda como root pelo systemctl, correto?). Isso evita depender de re-regras engessadas em sudoers e delega inteiramente a operação restrita ao servidor de orquestração.
- **Tratamento de Erros:** Vários erros do sistema reportam strings com `anyhow::Result` formatados até a tela do usuário. Criar uma tratativa própria `enum BiGameError` associada ao `thiserror` no `bigame-core` separaria as entranhas logadas do que é amigavelmente injetado no `adw::Toast` para o usuário na interface GTK.

## 3. Funcionalidades Potenciais
- **Gráficos de Tempo-Real / Benchmark:** Injetar uma view lateral interativa exibindo estatísticas ao vivo do sistema (uso de CPU, GPU, Temperaturas). O usuário saberá ativamente que o modo Gamer realmente aplicou melhorias em tempo real!
- **Auto-Start por Jogo (Perfis por Processos):** Criar integração entre a checagem de execução e a captura de chamadas `bwrap` (Steam, Lutris, Heroic), detectando automaticamente a liberação de um jogo e aplicando o perfil Gamer no modo automático.
- **Sincronia na Nuvem com BigLinux ID:** Integração para o usuário manter backups dos seus perfis salvos, recuperando suas customizações perfeitamente ao reinstalar o sistema.

## 4. Estrutura e Empacotamento
- **CI de Automação:** Criar uma branch de GitHub Actions `.github/workflows/rust.yml` que valide a syntax através do `cargo clippy --pedantic` e efetue binds em pacotes para verificar brechas e code debt a cada PR.

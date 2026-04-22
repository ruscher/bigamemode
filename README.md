# 🎮 BiGame-mode

**Dynamic Gaming Performance Orchestration for BigLinux**

BiGame-mode is a comprehensive gaming optimization suite designed to squeeze every frame out of your hardware. It unifies high-performance scheduling, GPU optimization, and micro-compositing into a single, elegant Libadwaita interface.

## 📖 A História por Trás do Projeto

Eu, **Rafael Ruscher**, sempre fui apaixonado por jogos. Sou um grande entusiasta e, principalmente, um defensor ferrenho de jogos no Linux. Nos últimos anos, vimos o jogo virar: com as melhorias constantes e o apoio massivo da **Valve**, a compatibilidade hoje é quase total.

Fico extremamente feliz em poder jogar com amigos como o **Barnabé di Kartola**, e acompanhar a turma do **Alessandro** e do **Pacheco** do canal **System Infotech**. Eles jogam diariamente e, sempre que me sobra um tempinho, estou lá jogando com eles. Ver canais mostrando o **BigLinux** em ação me motiva profundamente.

Em respeito a essa comunidade e para garantir que todos tenham a melhor experiência possível, criei o **BiGame-mode**. O objetivo é aproveitar o máximo do hardware, trazendo os últimos recursos tecnológicos para alcançar o FPS máximo. Com a integração do `lsfg-vk` (Lossless Scaling) e o `falcond`, criamos uma solução completa de GameMode para o ecossistema BigLinux.

---

## 🛠️ Visão Geral Técnica

O BiGame-mode atua como a central de controle e orquestração, gerenciando diversos componentes de baixo nível do sistema:

### 1. falcond (Dependência Externa)
O daemon de performance escrito em Zig que monitora o estado do sistema e aplica os perfis.
- **Integração Sched-ext**: Alterna dinamicamente entre escalonadores BPF (`bpfland`, `lavd`, etc.).
- **Gestão de VCache**: Otimiza a alocação de cache em processadores AMD Ryzen 3D.
- **Controle de Governador CPU**: Alterna entre modos `performance` e `powersave`.

### 2. lsfg-vk (Dependência Externa)
Integração com a camada Vulkan de Lossless Scaling Frame Generation.
- Fornece escalonamento de alta qualidade e geração de quadros para maior fluidez.

### 3. Gamescope & GameMode
A interface também orquestra o micro-compositor Gamescope e o GameMode da Feral Interactive, garantindo que todas as ferramentas de performance do Linux trabalhem em harmonia.

---

## 🚀 Funcionalidades

- **Wizard de Perfil**: Guia passo a passo para iniciantes (explicando tecnologias complexas com analogias simples).
- **Dashboard**: Telemetria em tempo real de frequências, temperaturas e otimizações ativas.
- **Gestão de Perfis**: Overrides por jogo para escalonadores, scripts e configurações de tela.
- **Integração com Tray**: Troca rápida de perfis e monitoramento via ícone de sistema.
- **Interface Glassmorphism**: Experiência visual premium seguindo a estética do BigLinux.

---

## 📦 Instalação

### Arch Linux / Manjaro / BigLinux
Certifique-se de ter os repositórios do BigLinux ativos para as dependências `falcond` e `lsfg-vk`.

```bash
# Clonar o repositório da interface
git clone https://github.com/ruscher/bigamemode.git
cd bigamemode/pkgbuild
makepkg -si
```


### Nix / NixOS
```bash
nix profile install github:biglinux/bigamemode
```

---

## 🤝 Credits

- **Developer**: Rafael Ruscher (rruscher@gmail.com)
- **Special Thanks**:
  - Bruno Gonçalves
  - Barnabé di Kartola
  - Alessandro (System Infotech)
  - Pacheco (System Infotech)
  - The BigLinux Community

---

## 📜 License

GPL-3.0-or-later

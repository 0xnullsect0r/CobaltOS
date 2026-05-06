# CobaltOS default ~/.zshrc
# Loaded for all interactive zsh sessions.

# ---------- History ----------
HISTFILE="$HOME/.zsh_history"
HISTSIZE=50000
SAVEHIST=50000
setopt SHARE_HISTORY HIST_IGNORE_DUPS HIST_REDUCE_BLANKS EXTENDED_HISTORY
setopt HIST_VERIFY    # expand history before running

# ---------- Options ----------
setopt AUTO_CD GLOB_DOTS NO_BEEP INTERACTIVE_COMMENTS CORRECT_ALL
setopt PUSHD_IGNORE_DUPS PUSHD_SILENT

# ---------- Completion ----------
autoload -Uz compinit && compinit -C
zstyle ':completion:*' menu select
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}'
zstyle ':completion:*' list-colors "${(s.:.)LS_COLORS}"
zstyle ':completion:*:*:kill:*' menu yes select
zstyle ':completion:*:kill:*' force-list always

# ---------- Keybindings ----------
bindkey -e                          # Emacs-style key bindings
bindkey '^[[A' history-search-backward   # Up arrow: history search
bindkey '^[[B' history-search-forward    # Down arrow: history search
bindkey '^[[1;5C' forward-word           # Ctrl+Right
bindkey '^[[1;5D' backward-word          # Ctrl+Left

# ---------- Plugins ----------
# zsh-autosuggestions
ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=240'
ZSH_AUTOSUGGEST_STRATEGY=(history completion)
[[ -f /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh ]] && \
    source /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh

# zsh-syntax-highlighting (must be last plugin)
[[ -f /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh ]] && \
    source /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh

# ---------- Rust CLI aliases ----------
if command -v rg  &>/dev/null; then alias grep='rg'; fi
if command -v fd  &>/dev/null; then alias find='fd'; fi
if command -v eza &>/dev/null; then
    alias ls='eza --icons --group-directories-first'
    alias ll='eza --icons --group-directories-first -l --git'
    alias la='eza --icons --group-directories-first -la --git'
    alias lt='eza --icons --tree --level=2'
    alias llt='eza --icons --tree --level=3 -l'
fi
if command -v bat    &>/dev/null; then
    alias cat='bat --pager=never'
    alias less='bat'
    export MANPAGER="sh -c 'col -bx | bat -l man -p'"
fi
if command -v dust   &>/dev/null; then alias du='dust'; fi
if command -v btm    &>/dev/null; then alias top='btm'; alias htop='btm'; fi
if command -v procs  &>/dev/null; then alias ps='procs'; fi
if command -v sd     &>/dev/null; then alias sed='sd'; fi
if command -v gping  &>/dev/null; then alias ping='gping'; fi
if command -v xh     &>/dev/null; then alias curl='xh'; fi
if command -v hexyl  &>/dev/null; then alias xxd='hexyl'; fi

# ---------- zoxide (smart cd) ----------
if command -v zoxide &>/dev/null; then
    eval "$(zoxide init zsh --cmd cd)"
fi

# ---------- Starship prompt ----------
if command -v starship &>/dev/null; then
    eval "$(starship init zsh)"
fi

# ---------- Convenience aliases ----------
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias cls='clear'
alias q='exit'
alias :q='exit'

# CobaltOS-specific
alias update='sudo cobalt-update apply'
alias cb-probe='sudo cobalt-hardware-probe'
alias cb-probe-audio='sudo cobalt-hardware-probe --fix-audio'

# git shortcuts
alias g='git'
alias gs='git status'
alias gd='git diff'
alias gl='git log --oneline --graph --decorate'
alias gc='git commit'
alias gp='git push'

# ---------- Environment ----------
export EDITOR='vim'
export VISUAL='cosmic-edit'
export BROWSER='firefox-esr'
export LANG='en_US.UTF-8'
export LC_ALL='en_US.UTF-8'

# Add user local bin to PATH
[[ -d "$HOME/.local/bin" ]] && export PATH="$HOME/.local/bin:$PATH"
[[ -d "$HOME/.cargo/bin" ]] && export PATH="$HOME/.cargo/bin:$PATH"

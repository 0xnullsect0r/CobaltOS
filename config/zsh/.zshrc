# CobaltOS default ~/.zshrc
# Loaded for all interactive zsh sessions.

# ---------- History ----------
HISTFILE="$HOME/.zsh_history"
HISTSIZE=50000
SAVEHIST=50000
setopt SHARE_HISTORY HIST_IGNORE_DUPS HIST_REDUCE_BLANKS EXTENDED_HISTORY

# ---------- Options ----------
setopt AUTO_CD GLOB_DOTS NO_BEEP INTERACTIVE_COMMENTS

# ---------- Completion ----------
autoload -Uz compinit && compinit
zstyle ':completion:*' menu select
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}'

# ---------- Plugins ----------
# zsh-autosuggestions
ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=240'
[[ -f /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh ]] && \
    source /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh

# zsh-syntax-highlighting (must be last)
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
fi
if command -v bat    &>/dev/null; then alias cat='bat --pager=never'; fi
if command -v dust   &>/dev/null; then alias du='dust'; fi
if command -v btm    &>/dev/null; then alias top='btm'; htop() { btm; }; fi
if command -v procs  &>/dev/null; then alias ps='procs'; fi
if command -v sd     &>/dev/null; then alias sed='sd'; fi
if command -v gping  &>/dev/null; then alias ping='gping'; fi

# zoxide (smart cd)
if command -v zoxide &>/dev/null; then
    eval "$(zoxide init zsh)"
fi

# ---------- Starship prompt ----------
if command -v starship &>/dev/null; then
    eval "$(starship init zsh)"
fi

# ---------- Convenience ----------
alias ..='cd ..'
alias ...='cd ../..'
alias cls='clear'
alias update='sudo cobalt-update apply'
alias cb-probe='sudo cobalt-hardware-probe'

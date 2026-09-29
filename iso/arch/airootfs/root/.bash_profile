# ~/.bash_profile for SecureForge Live ISO root user
if [[ -f ~/.bashrc ]]; then
    . ~/.bashrc
fi

# Auto-launch SecureForge TUI Kiosk on tty1
if [[ "$(tty)" == "/dev/tty1" ]]; then
    clear
    echo -e "\e[1;38;2;56;189;248m┌────────────────────────────────────────────────────────┐\e[0m"
    echo -e "\e[1;38;2;56;189;248m│  🔒 SECUREFORGE FORENSIC WORKSTATION (LIVE BOOT)       │\e[0m"
    echo -e "\e[1;38;2;56;189;248m└────────────────────────────────────────────────────────┘\e[0m"
    echo -e "\e[38;2;148;163;184mStarting SecureForge Terminal Interface...\e[0m\n"
    
    # Launch TUI
    if [[ -x /usr/local/bin/sforge ]]; then
        /usr/local/bin/sforge
    elif [[ -x /usr/bin/sforge ]]; then
        /usr/bin/sforge
    fi

    echo -e "\n\e[1;38;2;52;211;153m[i] SecureForge session closed.\e[0m"
    echo -e "    Type \e[1msforge\e[0m to restart the TUI interface."
    echo -e "    Type \e[1msih149 --help\e[0m for forensic CLI commands."
    echo -e "    Type \e[1msih149 demo\e[0m to run autonomous verification demo."
    echo -e "    Web Workstation active on \e[1;38;2;56;189;248mhttp://localhost:7878\e[0m\n"
fi

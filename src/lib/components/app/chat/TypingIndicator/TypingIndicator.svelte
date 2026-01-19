<div class="typing-indicator">
    <span></span><span></span><span></span>
</div>

<style>
    .typing-indicator {
        display: flex;
        gap: 4px;
        align-items: center;
    }

    .typing-indicator span {
        width: 8px;
        height: 8px;
        background-color: var(--color-accent);
        border-radius: 50%;
        animation: bounce 1.4s infinite;
    }

    .typing-indicator span:nth-child(2) {
        animation-delay: 0.2s;
    }

    .typing-indicator span:nth-child(3) {
        animation-delay: 0.4s;
    }

    @keyframes bounce {
        0%, 80%, 100% {
            opacity: 0.5;
            transform: translateY(0);
        }
        40% {
            opacity: 1;
            transform: translateY(-10px);
        }
    }
</style>

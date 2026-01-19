<script>
    /** @type {{ role: 'user' | 'assistant' | 'system' }} */
    let { role } = $props();
</script>

<div class="avatar-small {role}">
    {role === "user" ? "U" : role === "system" ? "⚙" : "AI"}
</div>

<style>
    .avatar-small {
        width: 30px;
        height: 30px;
        border-radius: 2px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 12px;
        font-weight: bold;
    }

    .user {
        background-color: #5436da;
        color: white;
    }

    .assistant {
        background-color: var(--color-accent);
        color: white;
    }

    .system {
        background-color: #3b82f6;
        color: white;
    }
</style>

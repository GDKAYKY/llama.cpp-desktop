import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { chatStore } from '../src/lib/stores/chat.svelte';

describe('Chat Store Logic (Tauri IPC)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    chatStore.messages = [];
    chatStore.isLoading = false;
  });

  it('should send a message and update the store with the response', async () => {
    // Simulate model loaded state
    chatStore.modelLoaded = true;
    
    // Mock the response from Tauri
    const mockResponse = "Mocked AI Response";
    (invoke as any).mockResolvedValue(mockResponse);

    // Call the send function
    const messageContent = "Tell me a joke";
    const sendPromise = chatStore.send(messageContent);

    // Check loading state
    expect(chatStore.isLoading).toBe(true);
    expect(chatStore.messages.length).toBe(1);
    expect(chatStore.messages[0].content).toBe(messageContent);

    await sendPromise;

    // Check final state
    expect(chatStore.isLoading).toBe(false);
    expect(chatStore.messages.length).toBe(2);
    expect(chatStore.messages[1].role).toBe('assistant');
    expect(chatStore.messages[1].content).toBe(mockResponse);
    
    // Check that Tauri was called correctly
    expect(invoke).toHaveBeenCalledWith('send_message', { message: messageContent });
  });

  it('should handle errors when the model fails to respond', async () => {
    chatStore.modelLoaded = true;
    (invoke as any).mockRejectedValue(new Error("Model failed"));

    await chatStore.send("Hello");

    expect(chatStore.isLoading).toBe(false);
    expect(chatStore.error).toBe("Model failed");
    expect(chatStore.messages.length).toBe(1); // Only user message should be there
  });
});

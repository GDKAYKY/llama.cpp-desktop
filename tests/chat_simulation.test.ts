import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { invoke } from '@tauri-apps/api/core';
import ChatForm from '../src/lib/components/chat/ChatForm.svelte';
import { chatStore } from '../src/lib/stores/chat.svelte';

// Mock the chat store or let it use the mocked invoke
// In this case, chatStore.send calls invokeCommand('send_message', ...)

describe('Chat UI Simulation', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    chatStore.messages = [];
    chatStore.isLoading = false;
  });

  it('should simulate sending a message and receiving a response from the model', async () => {
    // 1. Simulating model loading (already loaded in chatStore by default for this test)
    chatStore.modelLoaded = true;
    
    // 2. Mocking the Tauri 'send_message' command response
    const mockResponse = "Hello! I am the llama model. How can I help you today?";
    (invoke as any).mockResolvedValue(mockResponse);

    // 3. Rendering the ChatForm (or a portion of the UI)
    // For a more complete test, we could render the whole page, but let's focus on the interaction
    let userInput = "";
    const onSend = async () => {
      if (userInput.trim()) {
        const content = userInput;
        userInput = "";
        await chatStore.send(content);
      }
    };

    const { getByPlaceholderText, getByRole } = render(ChatForm, {
      props: {
        userInput: userInput,
        modelLoaded: chatStore.modelLoaded,
        isLoading: chatStore.isLoading,
        onSend: onSend,
        onInput: () => {},
        onKeydown: (e: any) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            onSend();
          }
        }
      }
    });

    const textarea = getByPlaceholderText('Message llama-desktop...');
    const sendButton = getByRole('button', { name: /send message/i });

    // 4. Simulate user typing
    await fireEvent.input(textarea, { target: { value: 'Hello Llama!' } });
    // Note: Since we are using bindable props in Svelte 5, we might need to manually update if testing purely isolated component
    // But actually chatStore.send is what we want to test.
    
    // For the sake of this simulation, let's trigger the send directly
    await fireEvent.click(sendButton);

    // 5. Verify the request was sent to Tauri
    expect(invoke).toHaveBeenCalledWith('send_message', { message: 'Hello Llama!' });

    // 6. Verify the assistant message is added to the store after the promise resolves
    await waitFor(() => {
      expect(chatStore.messages.length).toBe(2);
      expect(chatStore.messages[1].role).toBe('assistant');
      expect(chatStore.messages[1].content).toBe(mockResponse);
    });
    
    console.log('Simulated Chat Flow:');
    console.log('User:', chatStore.messages[0].content);
    console.log('Model Response:', chatStore.messages[1].content);
  });
});

import { invoke } from '@tauri-apps/api/core';
import type {
    Message,
    GenerationParams,
    DEFAULT_GENERATION_PARAMS,
} from '../types/backend';

export class OrchestratorService {
    /**
     * Create a new conversation slot
     */
    static async createSlot(maxCtx: number = 10): Promise<string> {
        return invoke('create_slot', { max_ctx: maxCtx });
    }

    /**
     * Delete a conversation slot
     */
    static async deleteSlot(slotId: string): Promise<boolean> {
        return invoke('delete_slot', { slot_id: slotId });
    }

    /**
     * List all active conversation slots
     */
    static async listSlots(): Promise<string[]> {
        return invoke('list_slots');
    }

    /**
     * Get all messages in a conversation
     */
    static async getSlotMessages(slotId: string): Promise<Message[]> {
        return invoke('get_slot_messages', { slot_id: slotId });
    }

    /**
     * Send a message and get response (non-streaming)
     */
    static async sendMessage(
        slotId: string,
        message: string,
        params?: GenerationParams
    ): Promise<string> {
        return invoke('send_message', {
            slot_id: slotId,
            message,
            params,
        });
    }

    /**
     * Clear all messages in a conversation
     */
    static async clearSlot(slotId: string): Promise<void> {
        return invoke('clear_slot', { slot_id: slotId });
    }

    /**
     * Get conversation statistics
     */
    static async getStats(): Promise<{
        totalSlots: number;
        slots: Array<{ id: string; messageCount: number }>;
    }> {
        const slots = await this.listSlots();
        const slotStats = await Promise.all(
            slots.map(async (id) => ({
                id,
                messageCount: (await this.getSlotMessages(id)).length,
            }))
        );

        return {
            totalSlots: slots.length,
            slots: slotStats,
        };
    }
}

/**
 * Hook for managing a single conversation
 */
export function useConversation() {
    let slotId: string | null = null;
    let messages: Message[] = [];

    return {
        async init(maxCtx?: number) {
            slotId = await OrchestratorService.createSlot(maxCtx);
            messages = [];
            return slotId;
        },

        async send(message: string, params?: GenerationParams) {
            if (!slotId) throw new Error('Conversation not initialized');
            const response = await OrchestratorService.sendMessage(
                slotId,
                message,
                params
            );
            messages = await OrchestratorService.getSlotMessages(slotId);
            return response;
        },

        async getMessages() {
            if (!slotId) return [];
            messages = await OrchestratorService.getSlotMessages(slotId);
            return messages;
        },

        async clear() {
            if (!slotId) return;
            await OrchestratorService.clearSlot(slotId);
            messages = [];
        },

        async destroy() {
            if (!slotId) return;
            await OrchestratorService.deleteSlot(slotId);
            slotId = null;
            messages = [];
        },

        getSlotId() {
            return slotId;
        },
    };
}

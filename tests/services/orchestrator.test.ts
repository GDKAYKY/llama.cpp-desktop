import { describe, it, expect, vi } from 'vitest';
import { OrchestratorService } from '$lib/services/orchestrator';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

describe('OrchestratorService', () => {
    it('createSlot calls invoke with correct params', async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        vi.mocked(invoke).mockResolvedValue('slot-123');
        
        const slotId = await OrchestratorService.createSlot(10);
        
        expect(invoke).toHaveBeenCalledWith('create_slot', { max_ctx: 10 });
        expect(slotId).toBe('slot-123');
    });

    it('deleteSlot calls invoke with slot_id', async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        vi.mocked(invoke).mockResolvedValue(true);
        
        const result = await OrchestratorService.deleteSlot('slot-123');
        
        expect(invoke).toHaveBeenCalledWith('delete_slot', { slot_id: 'slot-123' });
        expect(result).toBe(true);
    });

    it('listSlots returns array of slot IDs', async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        vi.mocked(invoke).mockResolvedValue(['slot-1', 'slot-2']);
        
        const slots = await OrchestratorService.listSlots();
        
        expect(invoke).toHaveBeenCalledWith('list_slots');
        expect(slots).toEqual(['slot-1', 'slot-2']);
    });

    it('getSlotMessages returns messages array', async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        const messages = [
            { role: 'user', content: 'Hello' },
            { role: 'assistant', content: 'Hi' },
        ];
        vi.mocked(invoke).mockResolvedValue(messages);
        
        const result = await OrchestratorService.getSlotMessages('slot-123');
        
        expect(invoke).toHaveBeenCalledWith('get_slot_messages', { slot_id: 'slot-123' });
        expect(result).toEqual(messages);
    });

    it('clearSlot calls invoke', async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        vi.mocked(invoke).mockResolvedValue(undefined);
        
        await OrchestratorService.clearSlot('slot-123');
        
        expect(invoke).toHaveBeenCalledWith('clear_slot', { slot_id: 'slot-123' });
    });

    it('getStats aggregates slot information', async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        vi.mocked(invoke)
            .mockResolvedValueOnce(['slot-1', 'slot-2'])
            .mockResolvedValueOnce([{ role: 'user', content: 'msg1' }])
            .mockResolvedValueOnce([{ role: 'user', content: 'msg2' }, { role: 'assistant', content: 'msg3' }]);
        
        const stats = await OrchestratorService.getStats();
        
        expect(stats.totalSlots).toBe(2);
        expect(stats.slots).toHaveLength(2);
        expect(stats.slots[0].messageCount).toBe(1);
        expect(stats.slots[1].messageCount).toBe(2);
    });
});
